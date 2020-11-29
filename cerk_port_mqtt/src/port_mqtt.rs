use anyhow::{bail, Context, Result};
use async_std::task::block_on;
use cerk::kernel::{
    BrokerEvent, CloudEventMessageRoutingId, CloudEventRoutingArgs, Config, DeliveryGuarantee,
    IncomingCloudEvent, OutgoingCloudEvent, OutgoingCloudEventProcessed, ProcessingResult,
};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerFnRefStatic, InternalServerId};
use cloudevents::{AttributesReader, Event};
use paho_mqtt::{
    AsyncClient, ConnectOptionsBuilder, CreateOptions, CreateOptionsBuilder, Message,
    PersistenceType,
};
use serde_json;
use std::future::Future;
use std::rc::Rc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use nix::unistd::{fork, ForkResult};
use interprocess::unnamed_pipe::{pipe, UnnamedPipeWriter, UnnamedPipeReader};
use std::io::{BufReader, BufRead};
use serde::{Serialize, Deserialize};
use std::io::Write;
use std::fmt;
use std::sync::{Arc,RwLock};
use std::thread;

struct MqttConnection {
    client: AsyncClient,
    processed_tx: Sender<ProcessingResult>,
    send_topic: Option<String>,
    send_qos: u8,
    subscribe_topic: Option<String>,
    subscribe_qos: u8,
}

#[derive(Serialize, Deserialize, Debug)]
enum IpcEvent {
    Init,
    ConfigUpdated(Config),
    OutgoingCloudEvent(OutgoingCloudEvent),
    OutgoingCloudEventProcessed(OutgoingCloudEventProcessed),
    IncomingCloudEvent(IncomingCloudEvent),
    IncomingCloudEventProcessed(CloudEventMessageRoutingId, ProcessingResult),
}

fn build_connection(
    id: &InternalServerId,
    config: Config,
    processed_tx: Sender<ProcessingResult>,
) -> MqttConnection {
    match config {
        Config::HashMap(ref config_map) => {
            let host = match config_map.get("host") {
                Some(Config::String(host)) => host,
                _ => panic!("{} invalid value for host", id),
            };

            let send_topic = match config_map.get("send_topic") {
                Some(Config::String(topic)) => Some(topic.clone()),
                Some(_) => panic!("{} invalid value for send_topic", id),
                _ => None,
            };

            let send_qos = match config_map.get("send_qos") {
                Some(Config::U8(qos)) => *qos,
                Some(_) => panic!("{} invalid value for send_qos", id),
                _ => 0,
            };

            let subscribe_topic = match config_map.get("subscribe_topic") {
                Some(Config::String(topic)) => Some(topic.clone()),
                Some(_) => panic!("{} invalid value for subscribe_topic", id),
                _ => None,
            };

            let subscribe_qos = match config_map.get("subscribe_qos") {
                Some(Config::U8(qos)) => *qos,
                Some(_) => panic!("{} invalid value for subscribe_qos", id),
                _ => 0,
            };

            let mqtt_config = CreateOptionsBuilder::new()
                .client_id(format!("cerk-{}", id))
                .server_uri(host)
                .persistence(PersistenceType::None)
                .mqtt_version(5)
                .finalize();

            let client = AsyncClient::new(mqtt_config).unwrap_or_else(|err| {
                panic!("Error creating the client: {}", err);
            });

            return MqttConnection {
                client,
                processed_tx,
                send_topic,
                send_qos,
                subscribe_topic,
                subscribe_qos,
            };
        }
        _ => panic!("{} received invalide config", id),
    }
}

fn message_handler(
    id: InternalServerId,
    processed_rx: Receiver<ProcessingResult>,
    parent_tx: Arc<RwLock<UnnamedPipeWriter>>,
    routing_args: CloudEventRoutingArgs,
) -> Box<dyn Fn(&AsyncClient, Option<paho_mqtt::Message>)> {
    Box::new(
        move |_client: &AsyncClient, msg: Option<paho_mqtt::Message>| {
            debug!("{} received message callback", id);
            if let Some(msg) = msg {
                debug!("{} received cloudevent on topic {}", id, msg.topic());
                let payload_str = msg.payload_str();
                match serde_json::from_str::<Event>(&payload_str) {
                    Ok(cloud_event) => {
                        debug!("{} deserialized event successfully", id);
                        // todo add delivery attempt to routing id
                        let routing_id = cloud_event.id().to_string();
                        send_to_process(&mut parent_tx.write().unwrap(), IpcEvent::IncomingCloudEvent(
                            IncomingCloudEvent {
                                routing_id,
                                incoming_id: id.clone(),
                                cloud_event,
                                args: routing_args.clone(),
                            }
                        )).unwrap();
                        processed_rx.recv();
                    }
                    Err(err) => {
                        error!("{} while converting string to CloudEvent: {:?}", id, err);
                    }
                }
            }
        },
    )
}

async fn setup_connection(
    id: &InternalServerId,
    parent_tx: Arc<RwLock<UnnamedPipeWriter>>,
    config: Config,
) -> Result<MqttConnection> {
    debug!("{} start connection to mqtt broker", id);

    let (processed_tx, processed_rx) = channel();
    let mut connection = build_connection(id, config, processed_tx);

    let connection_options = ConnectOptionsBuilder::new()
        .clean_session(false)
        .clean_start(false)
        .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(5))
        .finalize();

    connection.client.connect(connection_options).await?;

    let routing_args = CloudEventRoutingArgs {
        delivery_guarantee: match connection.subscribe_qos {
            1 => DeliveryGuarantee::AtLeastOnce,
            _ => DeliveryGuarantee::Unspecified,
        },
    };

    connection.client.set_message_callback(message_handler(
        id.clone(),
        processed_rx,
        parent_tx,
        routing_args,
    ));

    if let Some(ref subscribe_topic) = connection.subscribe_topic {
        debug!(
            "{} subscribes to {} with qos {}",
            id, subscribe_topic, connection.subscribe_qos,
        );

        connection
            .client
            .subscribe(subscribe_topic, connection.subscribe_qos as i32)
            .await?;
    }

    return Ok(connection);
}

async fn send_cloud_event(
    id: &InternalServerId,
    cloud_event: &Event,
    connection: &MqttConnection,
) -> Result<ProcessingResult> {
    if let Some(ref send_topic) = connection.send_topic {
        let serialized = serde_json::to_string(cloud_event).unwrap();
        debug!("{} message serialized", id);
        let msg = Message::new(send_topic, serialized, connection.send_qos as i32);
        debug!("start publishing on {}", send_topic);

        match connection.client.publish(msg).await {
            Ok(_) => Ok(ProcessingResult::Successful),
            Err(e) => {
                error!("{} error while publishing {:?}", id, e);
                Ok(ProcessingResult::PermanentError)
            }
        }
    } else {
        bail!(
            "{} received event before the mqtt port was configured as output port -> message will be dropped",
            id
        )
    }
}

fn send_to_process(process_tx: &mut UnnamedPipeWriter, ipc_event: IpcEvent) -> Result<()> {
    let event_json = serde_json::to_string(&ipc_event)?;
    write!(process_tx, "{}\n", event_json)?;
    Ok(())
}

fn backchannel(sender_to_kernel: BoxedSender, mut parent_rx: UnnamedPipeReader) {
    let reader = BufReader::new(parent_rx)
        .lines()
        .map(|line| line.unwrap())
        .map(|line| serde_json::from_str::<IpcEvent>(&line).unwrap());

    for ipc_event in reader {
        debug!("Event received from CHILD PROCESS {:?}", ipc_event);
        match ipc_event {
            IpcEvent::IncomingCloudEvent(event) => sender_to_kernel.send(BrokerEvent::IncomingCloudEvent(event)),
            IpcEvent::OutgoingCloudEventProcessed(event) => sender_to_kernel.send(BrokerEvent::OutgoingCloudEventProcessed(event)),
            _ => panic!("Unexpected event in backchannels {:?}", ipc_event),
        }
    }
}

async fn port_mqtt_start_parent(id: InternalServerId, inbox: BoxedReceiver, sender_to_kernel: BoxedSender, mut parent_rx: UnnamedPipeReader, mut child_tx: UnnamedPipeWriter) -> Result<()> {
    thread::spawn(move || {
        backchannel(sender_to_kernel, parent_rx)
    });
    
    loop {
        let broker_event = inbox.receive();

        match broker_event {
            BrokerEvent::Init => {
                info!("PARENT PROCESS {} initiated", id);
                send_to_process(&mut child_tx, IpcEvent::Init)?;
            }
            BrokerEvent::ConfigUpdated(config, _) => {
                info!("PARENT PROCESS {} received ConfigUpdated", &id);
                send_to_process(&mut child_tx, IpcEvent::ConfigUpdated(config))?;
            }
            BrokerEvent::OutgoingCloudEvent(event) => {
                debug!("PARENT PROCESS {} cloudevent received", &id);
                send_to_process(&mut child_tx, IpcEvent::OutgoingCloudEvent(event))?;
            }
            BrokerEvent::IncomingCloudEventProcessed(event_id, result) => {
                debug!("PARENT PROCESS {} message {} processed -> {}", id, event_id, result);
                send_to_process(&mut child_tx, IpcEvent::IncomingCloudEventProcessed(event_id, result))?;
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

async fn port_mqtt_start_child(id: InternalServerId, mut child_rx: UnnamedPipeReader, mut parent_tx: UnnamedPipeWriter) -> Result<()> {
    let mut connection: Option<MqttConnection> = None;
    let mut parent_tx_arc = Arc::new(RwLock::new(parent_tx));
    let reader = BufReader::new(child_rx)
        .lines()
        .map(|line| line.unwrap())
        .map(|line| serde_json::from_str::<IpcEvent>(&line).unwrap());
    for ipc_event in reader {
        debug!("Event received from PARENT PROCESS {:?}", ipc_event);

        match ipc_event {
            IpcEvent::Init => {
                info!("CHILD PROCESS {}: initiated", id);
            }
            IpcEvent::ConfigUpdated(config) => {
                info!("CHILD PROCESS {} received ConfigUpdated", &id);

                if let Some(ref connection) = connection {
                    match connection.client.disconnect(None).await {
                        Ok(_) => debug!("disconnected succesfully"),
                        Err(err) => panic!("{} disconnects failed {:?}", id, err),
                    }
                }

                match setup_connection(
                    &id,
                    parent_tx_arc.clone(),
                    config,
                ).await {
                    Ok(new_connection) => {
                        connection = Some(new_connection);
                    }
                    Err(err) => panic!("{} connection setup failed {:?}", id, err),
                }
            }
            IpcEvent::OutgoingCloudEvent(event) => {
                debug!("CHILD PROCESS {} cloudevent received", &id);

                if let Some(ref connection) = connection {
                    match send_cloud_event(&id, &event.cloud_event, &connection).await {
                        Ok(result) => {
                            debug!("CHILD PROCESS {} cloudevent sent -> {:?}", &id, &result);
                            
                            send_to_process(&mut parent_tx_arc.write().unwrap(), IpcEvent::OutgoingCloudEventProcessed(
                                OutgoingCloudEventProcessed {
                                    sender_id: id.clone(),
                                    routing_id: event.routing_id,
                                    result: result,
                                }
                            ))?;
                        }
                        Err(err) => panic!("CHILD PROCESS {} connection setup failed {:?}", id, err),
                    }
                } else {
                    panic!("CHILD PROCESS {} can not send message, no connection configured", id)
                }
            }
            IpcEvent::IncomingCloudEventProcessed(event_id, result) => {
                debug!("CHILD PROCESS {} message {} processed -> {}", id, event_id, result);

                if let Some(ref connection) = connection {
                    match result {
                        ProcessingResult::Successful => {
                            connection.processed_tx.send(result).unwrap()
                        }
                        _ => panic!("CHILD PROCESS {} message processing failed, restart", id),
                    }
                } else {
                    panic!("CHILD PROCESS {} can not send message, no connection configured", id)
                }
            }
            ipc_event => warn!("event {:?} not implemented", ipc_event),
        }
    }
    Ok(())
}

/// This is the main function to start the port.
pub fn port_mqtt_start(id: InternalServerId, inbox: BoxedReceiver, sender_to_kernel: BoxedSender) {
    mitosis::init();
    let (mut parent_tx, mut parent_rx) = pipe().unwrap();
    let (mut child_tx, mut child_rx) = pipe().unwrap();

    match unsafe{fork()} {
        Ok(ForkResult::Parent { child, .. }) => {
            println!("Continuing execution in parent process, new child has pid: {}", child);
            block_on(port_mqtt_start_parent(id.clone(), inbox, sender_to_kernel, parent_rx, child_tx)).unwrap();
        }
        Ok(ForkResult::Child) => {
            println!("I'm a new child process");
            block_on(port_mqtt_start_child(id.clone(), child_rx, parent_tx)).unwrap();
        },
        Err(_) => println!("Fork failed"),
     }

    info!("start mqtt port with id {}", id);
}

/// This is the pointer for the main function to start the port.
pub static PORT_MQTT: InternalServerFnRefStatic = &(port_mqtt_start as InternalServerFn);

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn setup_connection_with_minimal_send_config() {
        let map: HashMap<String, Config> = [
            (
                "host".to_string(),
                Config::String("tcp://mqtt-broker:1883".to_string()),
            ),
            ("send_topic".to_string(), Config::String("test".to_string())),
        ]
        .iter()
        .cloned()
        .collect();

        let config = Config::HashMap(map);

        let (_, options) = build_configs(&"test".to_string(), config);
        assert_eq!(options.send_topic, Some("test".to_string()));
        assert_eq!(options.send_qos, 0);
    }
    #[test]
    fn setup_connection_with_minimal_receive_config() {
        let map: HashMap<String, Config> = [
            (
                "host".to_string(),
                Config::String("tcp://mqtt-broker:1883".to_string()),
            ),
            (
                "subscribe_topics".to_string(),
                Config::Vec(vec![Config::String("test".to_string())]),
            ),
            (
                "subscribe_qos".to_string(),
                Config::Vec(vec![Config::U8(2)]),
            ),
        ]
        .iter()
        .cloned()
        .collect();

        let config = Config::HashMap(map);

        let (_, options) = build_configs(&"test".to_string(), config);
        assert_eq!(options.subscribe_topics, ["test".to_string()]);
        assert_eq!(options.subscribe_qos, [2]);
    }
    #[test]
    fn setup_connection_with_full_config() {
        let map: HashMap<String, Config> = [
            (
                "host".to_string(),
                Config::String("tcp://mqtt-broker:1883".to_string()),
            ),
            ("send_topic".to_string(), Config::String("test".to_string())),
            ("persistence".to_string(), Config::U8(0)),
            ("send_qos".to_string(), Config::U8(2)),
            (
                "subscribe_topics".to_string(),
                Config::Vec(vec![Config::String("test".to_string())]),
            ),
            (
                "subscribe_qos".to_string(),
                Config::Vec(vec![Config::U8(2)]),
            ),
        ]
        .iter()
        .cloned()
        .collect();

        let config = Config::HashMap(map);

        let (_, options) = build_configs(&"test".to_string(), config);
        assert_eq!(options.send_topic, Some("test".to_string()));
        assert_eq!(options.subscribe_topics, ["test".to_string()]);
        assert_eq!(options.subscribe_qos, [2]);
        assert_eq!(options.send_qos, 2);
    }
}
