use anyhow::{Context, Result};
use cerk::kernel::{
    BrokerEvent, CloudEventMessageRoutingId, CloudEventRoutingArgs, Config, ConfigHelpers,
    DeliveryGuarantee, IncomingCloudEvent, OutgoingCloudEvent, OutgoingCloudEventProcessed,
    ProcessingResult,
};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerFnRefStatic, InternalServerId};
use cloudevents::{AttributesReader, Event};
use mosquitto_client_wrapper::Mosquitto;
use serde_json;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use url::Url;

const MOSQ_OPT_DELAYED_ACK: u32 = 14;

struct Data {
    unacked: HashMap<i32, String>,
}

#[derive(Clone)]
struct Configurations {
    send_topic: Option<String>,
    subscribe_topic: Option<String>,
    subscribe_qos: u8,
    host_name: String,
    host_port: u16,
}

#[derive(Clone)]
struct Connection {
    client: Mosquitto,
    configs: Configurations,
}

type ArcData = Arc<Mutex<Data>>;

fn build_configurations(config: Config) -> Result<Configurations> {
    let host = config.get_op_val_string("host")?.unwrap();
    let send_topic = config.get_op_val_string("send_topic")?;
    let subscribe_topic = config.get_op_val_string("subscribe_topic")?;
    let subscribe_qos = config.get_op_val_u8("subscribe_qos")?.unwrap_or(0);

    if send_topic.is_some() && subscribe_topic.is_some() {
        bail!("received send_topic and subscribe_topic - only one is allowed!")
    }

    let host = Url::parse(&host)?;
    let host_name = host.host_str().ok_or(anyhow!("no host was provided"))?.to_string();
    let host_port = host.port().unwrap_or(1883);
    Ok(Configurations { send_topic, subscribe_topic, subscribe_qos, host_name, host_port })
}

/// checks if the configurations are valid for this port
pub fn check_configurations(config: Config) -> Result<()> {
    build_configurations(config)?;
    Ok(())
}

fn build_connection(id: &InternalServerId, config: Config) -> Result<Connection> {
    const RECONNECT_DELAY_MIN_SECONDS: u32 = 1;
    const RECONNECT_DELAY_MAX_SECONDS: u32 = 300;
    let configs = build_configurations(config)?;

    debug!("create new session: {}", id);
    info!("{} connect to: {}:{}", id, configs.host_name, configs.host_port);

    let client = Mosquitto::new_session(&id.clone(), false)?; // keep old session
    client.threaded()?;
    client.set_option(MOSQ_OPT_DELAYED_ACK, 1)?;
    client.reconnect_delay_set(
        RECONNECT_DELAY_MIN_SECONDS,
        RECONNECT_DELAY_MAX_SECONDS,
        true,
    )?;
    client.connect(configs.host_name.as_str(), configs.host_port.into(), 5)?;

    let connection = Connection {
        client,
        configs,
    };

    return Ok(connection);
}

fn connect(
    id: InternalServerId,
    connection: Connection,
    sender_to_kernel: BoxedSender,
    data: ArcData,
) -> Result<Sender<(CloudEventMessageRoutingId, ProcessingResult)>> {
    let (sender, receiver) = channel();
    let sub_delivery_guarantee = match connection.configs.subscribe_qos {
        1 => DeliveryGuarantee::AtLeastOnce,
        _ => DeliveryGuarantee::BestEffort,
    };
    thread::spawn(move || {
        let mut callbacks = connection.client.callbacks(Vec::<()>::new());
        callbacks.on_message(|_, msg| {
            let text = msg.text();
            debug!("received cloud event (on_message), text={}", text);
            let cloudevent: Event = serde_json::from_str(text).with_context(|| format!("{} failed to deserialize cloudevent {}", id, text)).unwrap();
            let routing_id = cloudevent.id().to_string();
            sender_to_kernel.send(BrokerEvent::IncomingCloudEvent(IncomingCloudEvent {
                incoming_id: id.clone(),
                routing_id: routing_id.clone(),
                cloud_event: cloudevent,
                args: CloudEventRoutingArgs {
                    delivery_guarantee: sub_delivery_guarantee,
                },
            }));
            if sub_delivery_guarantee.requires_acknowledgment() {
                debug!("ack required - block on_message");
                loop {
                    let (received_routing_id, result) = receiver.recv().unwrap();
                    if received_routing_id == routing_id {
                        debug!("received result for incoming cloud event: {}", result);
                        match result {
                            ProcessingResult::Successful => debug!("exiting on_message handler now"),
                            _ => panic!("message could not be forwarded, must prevent on_message from exiting (otherwise PUBACK would be sent)"),
                        }
                        return;
                    } else {
                        warn!("expected result for event: {}, got {}", routing_id, received_routing_id);
                    }
                }
            } else {
                debug!("no ack required - exit on_message");
            }
        });
        callbacks.on_publish(|_, message_id| {
            debug!("request lock for unacked messages list");
            let mut data_lock = data.lock().unwrap();
            debug!(
                "{} published message with id {} successfully",
                id, message_id
            );
            if let Some(routing_id) = data_lock.unacked.remove(&message_id) {
                send_processed_event(
                    id.clone(),
                    &sender_to_kernel,
                    routing_id,
                    ProcessingResult::Successful,
                );
            } else {
                warn!("on_publish {} was not expected", message_id)
            }
        });
        callbacks.on_connect(|_, connection_id| {
            debug!("{} connected: {}", id, connection_id);
            if let Some(ref subscribe_topic) = connection.configs.subscribe_topic {
                debug!(
                    "subscribe to: {} with qos {}",
                    subscribe_topic, connection.configs.subscribe_qos
                );
                connection
                    .client
                    .subscribe(&subscribe_topic, connection.configs.subscribe_qos.into())
                    .unwrap();
            }
        });
        callbacks.on_disconnect(|_, connection_id| {
            debug!("{} disconnected: {}", id, connection_id);
        });
        connection.client.loop_until_disconnect(200).unwrap();
    });

    return Ok(sender);
}

fn send_cloud_event(
    id: &InternalServerId,
    event: &OutgoingCloudEvent,
    connection: &Connection,
    data: ArcData,
) -> Result<()> {
    let serialized = serde_json::to_string(&event.cloud_event)?;

    if let Some(ref send_topic) = connection.configs.send_topic {
        let mut data_lock = data.lock().unwrap();
        let message_id = connection.client.publish(
            send_topic,
            serialized.as_bytes(),
            if event.args.delivery_guarantee.requires_acknowledgment() {
                1
            } else {
                0
            },
            false,
        )?;
        data_lock
            .unacked
            .insert(message_id, event.routing_id.clone());
        debug!("{} sent publish with id {}", id, message_id);
    } else {
        error!("{} not send_topic configured", id);
    }
    return Ok(());
}

fn send_processed_event(
    sender_id: InternalServerId,
    sender_to_kernel: &BoxedSender,
    routing_id: String,
    result: ProcessingResult,
) {
    sender_to_kernel.send(BrokerEvent::OutgoingCloudEventProcessed(
        OutgoingCloudEventProcessed {
            result,
            sender_id,
            routing_id,
        },
    ));
}

/// This is the main function to start the port.
pub fn port_mqtt_mosquitto_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    sender_to_kernel: BoxedSender,
) {
    info!("start mqtt port with id {}", id);
    let mut connection: Option<Connection> = None;
    let mut sender: Option<Sender<(CloudEventMessageRoutingId, ProcessingResult)>> = None;
    let data: ArcData = Arc::new(Mutex::new(Data {
        unacked: HashMap::new(),
    }));

    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                info!("{} initiated", id);
            }
            BrokerEvent::ConfigUpdated(config, _) => {
                info!("{} received ConfigUpdated", &id);
                connection = match build_connection(&id, config) {
                    Ok(new_connection) => Some(new_connection),
                    Err(e) => {
                        error!("failed to parse connection config {:?}", e);
                        None
                    }
                };
                if let Some(ref connection) = connection {
                    sender = match connect(
                        id.clone(),
                        connection.clone(),
                        sender_to_kernel.clone_boxed(),
                        data.clone(),
                    ) {
                        Ok(connection) => Some(connection),
                        Err(e) => {
                            error!("failed to connect {:?}", e);
                            None
                        }
                    };
                }
            }
            BrokerEvent::OutgoingCloudEvent(event) => {
                debug!("{} cloudevent received", &id);
                if let Some(ref connection) = connection {
                    debug!("{} will send event out", &id);
                    if let Err(e) = send_cloud_event(&id, &event, &connection, data.clone()) {
                        error!("failed to send event {:?}", e);
                        send_processed_event(
                            id.clone(),
                            &sender_to_kernel,
                            event.routing_id.clone(),
                            ProcessingResult::TransientError,
                        );
                    }
                } else {
                    error!("no active connection - can't send event");
                    send_processed_event(
                        id.clone(),
                        &sender_to_kernel,
                        event.routing_id.clone(),
                        ProcessingResult::TransientError,
                    );
                }
            }
            BrokerEvent::IncomingCloudEventProcessed(routing_id, result) => {
                if let Some(ref sender) = sender {
                    debug!(
                        "received IncomingCloudEventProcessed -> send result to on_message handler"
                    );
                    sender.send((routing_id, result)).unwrap();
                } else {
                    error!("no active connection - can't send result");
                }
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

/// This is the pointer for the main function to start the port.
pub static PORT_MQTT_MOSQUITTO: InternalServerFnRefStatic =
    &(port_mqtt_mosquitto_start as InternalServerFn);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_subscribe_config() {
        let map: HashMap<String, Config> = [
            ("host".to_string(), Config::String("tcp://mqtt-broker:1883".to_string())),
            ("send_topic".to_string(), Config::String("inbox".to_string())),
            ("subscribe_topic".to_string(), Config::String("outbox".to_string())),
            ("subscribe_qos".to_string(), Config::U8(1)),
        ]
            .iter()
            .cloned()
            .collect();
        assert!(check_configurations(Config::HashMap(map)).is_err());
    }
}
