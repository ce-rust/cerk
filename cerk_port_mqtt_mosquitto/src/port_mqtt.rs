use mosquitto_client::{Mosquitto,Callbacks};
use cerk::kernel::{BrokerEvent, CloudEventRoutingArgs, Config, IncomingCloudEvent, DeliveryGuarantee, OutgoingCloudEventProcessed, RoutingResult, ProcessingResult};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerFnRefStatic, InternalServerId};
use cloudevents::{AttributesReader, Event};
use serde_json;
use std::{thread,time};
use anyhow::Result;
use std::sync::mpsc::{channel,Sender};
use url::Url;

struct Connection {
    client: Mosquitto,
    send_topic: Option<String>,
    send_qos: u8,
    subscribe_topic: Option<String>,
    subscribe_qos: u8,
}

fn build_connection(id: &InternalServerId, config: Config) -> Result<Connection> {
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
            let host = Url::parse(host)?;
            debug!("create new session: {}", id);
            let client = mosquitto_client::Mosquitto::new_session(&id.clone(), false); // keep old session
            client.threaded();
            let host_name = host.host_str().unwrap();
            let host_port = host.port().unwrap_or(1883);
            debug!("connect to: {}:{}", host_name, host_port);
            client.connect(host_name, host_port.into(), 5)?;
            if let Some(ref subscribe_topic) = subscribe_topic {
                debug!("subscribe to: {} with qos {}", subscribe_topic, subscribe_qos);
                client.subscribe(&subscribe_topic, subscribe_qos.into())?;
            }
            

            let connection = Connection {
                client:client,
                send_topic: send_topic,
                send_qos: send_qos,
                subscribe_topic: subscribe_topic,
                subscribe_qos: subscribe_qos,
            };

            return Ok(connection);
        }
        _ => panic!("{} received invalide config", id),
    }
}

fn connect(id: InternalServerId, client: Mosquitto, sender_to_kernel: BoxedSender) -> Result<Sender<()>> {
    let (sender, receiver) = channel();
    thread::spawn(move || {
        let mut callbacks = client.callbacks(Vec::<()>::new());
        callbacks.on_message(|data,msg| {
            let cloudevent: Event = serde_json::from_str(msg.text()).unwrap();
            debug!("received cloud event (on_message)");
            sender_to_kernel.send(BrokerEvent::IncomingCloudEvent(IncomingCloudEvent{
                incoming_id:id.clone(),
                routing_id: "abc".to_string(),
                cloud_event: cloudevent,
                args: CloudEventRoutingArgs {
                    delivery_guarantee: DeliveryGuarantee::AtLeastOnce
                }
            }));
            debug!("wait for ack of cloud event - block");
            receiver.recv().unwrap();
            debug!("received ack for cloud event -> will ack to mqtt");
        });
        callbacks.on_publish(|data, id| {
            debug!("received on_publish {}", id);
        });
        client.loop_until_disconnect(200);
    });

    return Ok(sender);
}

fn send_cloud_event(
    id: &InternalServerId,
    cloud_event: &Event,
    connection: &Connection,
) -> Result<()> {
    let serialized = serde_json::to_string(cloud_event)?;
    // todo wait for publish confirm
    let id = if let Some(ref send_topic) = connection.send_topic {
        connection.client.publish(send_topic, serialized.as_bytes(), connection.send_qos.into(), false)?
    } else {
        bail!("no send_topic configured")
    };
    debug!("sent publish with id {}", id);
    return Ok(());
}


pub fn port_mqtt_mosquitto_start(id: InternalServerId, inbox: BoxedReceiver, sender_to_kernel: BoxedSender) {
    info!("start mqtt port with id {}", id);
    let mut connection: Option<Connection> = None;
    let mut sender: Option<Sender<()>> = None;

    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                info!("{} initiated", id);
            }
            BrokerEvent::ConfigUpdated(config, _) => {
                info!("{} received ConfigUpdated", &id);
                match build_connection(&id, config) {
                    Ok(new_connection) => connection = Some(new_connection),
                    Err(e) => error!("failed to connect {:?}", e)
                }

                if let Some(ref connection) = connection {
                    sender = Some(connect(id.clone(), connection.client.clone(), sender_to_kernel.clone_boxed()).unwrap());
                } else {
                    // TODO
                }
            }
            BrokerEvent::OutgoingCloudEvent(event) => {
                debug!("{} cloudevent received", &id);
                if let Some(ref connection) = connection {
                    debug!("{} will send event out", &id);
                    let result = send_cloud_event(&id, &event.cloud_event, &connection);
                    debug!("{} event sent out; successfull={}", &id, result.is_ok());
                    if event.args.delivery_guarantee.requires_acknowledgment() {
                        let process_result = match result {
                            Ok(_) => ProcessingResult::Successful,
                            Err(e) => {
                                error!("failed to publish message: {:?}", e);
                                ProcessingResult::PermanentError // todo permanent or transient
                            }
                        };
                        sender_to_kernel.send(BrokerEvent::OutgoingCloudEventProcessed(OutgoingCloudEventProcessed{
                            routing_id: event.routing_id,
                            result: process_result,
                            sender_id: id.clone(),
                        }))
                    }else {
                        debug!("no ack needed")
                    }
                } else {
                    error!("client is null - cant send event");
                }
            }
            BrokerEvent::IncomingCloudEventProcessed(event_id, result) => {
                // todo check result
                if let Some(ref sender) = sender {
                    debug!("received IncomingCloudEventProcessed -> will ack");
                    sender.send(()).unwrap();
                } else {
                    // TODO
                }
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

/// This is the pointer for the main function to start the port.
pub static PORT_MQTT_MOSQUITTO: InternalServerFnRefStatic = &(port_mqtt_mosquitto_start as InternalServerFn);
