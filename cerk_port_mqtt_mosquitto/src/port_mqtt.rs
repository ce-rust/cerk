use anyhow::Result;
use cerk::kernel::{
    BrokerEvent, CloudEventRoutingArgs, Config, DeliveryGuarantee, IncomingCloudEvent,
    OutgoingCloudEvent, OutgoingCloudEventProcessed, ProcessingResult, RoutingResult,
};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerFnRefStatic, InternalServerId};
use cloudevents::{AttributesReader, Event};
use mosquitto_client::{Callbacks, Mosquitto};
use serde_json;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::{thread, time};
use url::Url;

struct Data {
    unacked: HashMap<i32, String>,
}

#[derive(Clone)]
struct Connection {
    client: Mosquitto,
    send_topic: Option<String>,
    send_qos: u8,
    subscribe_topic: Option<String>,
    subscribe_qos: u8,
}

type ArcData = Arc<Mutex<Data>>;

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
            let host_name = host.host_str().unwrap();
            let host_port = host.port().unwrap_or(1883);

            debug!("create new session: {}", id);
            debug!("connect to: {}:{}", host_name, host_port);

            let client = mosquitto_client::Mosquitto::new_session(&id.clone(), false); // keep old session
            client.threaded();
            client.reconnect_delay_set(1, 300, true);
            client.connect(host_name, host_port.into(), 5)?;

            let connection = Connection {
                client: client,
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

fn connect(
    id: InternalServerId,
    connection: Connection,
    sender_to_kernel: BoxedSender,
    data: ArcData,
) -> Result<Sender<ProcessingResult>> {
    let (sender, receiver) = channel();
    let sub_delivery_guarantee = match connection.subscribe_qos {
        1 => DeliveryGuarantee::AtLeastOnce,
        _ => DeliveryGuarantee::Unspecified,
    };
    thread::spawn(move || {
        let mut callbacks = connection.client.callbacks(Vec::<()>::new());
        callbacks.on_message(|_, msg| {
            debug!("received cloud event (on_message)");
            let cloudevent: Event = serde_json::from_str(msg.text()).unwrap();
            sender_to_kernel.send(BrokerEvent::IncomingCloudEvent(IncomingCloudEvent {
                incoming_id: id.clone(),
                routing_id: cloudevent.id().to_string(),
                cloud_event: cloudevent,
                args: CloudEventRoutingArgs {
                    delivery_guarantee: sub_delivery_guarantee,
                },
            }));
            if sub_delivery_guarantee == DeliveryGuarantee::AtLeastOnce {
                debug!("ack required - block on_message");
                let result = receiver.recv().unwrap();
                debug!("received result for incomming cloud even: {}", result);
                match result {
                    ProcessingResult::Successful => debug!("exiting on_message handler now"),
                    _ => panic!("processing failed"),
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
                sender_to_kernel.send(BrokerEvent::OutgoingCloudEventProcessed(
                    OutgoingCloudEventProcessed {
                        result: ProcessingResult::Successful,
                        sender_id: id.clone(),
                        routing_id: routing_id,
                    },
                ));
            } else {
                warn!("on_publish {} was not expected", message_id)
            }
        });
        callbacks.on_connect(|_, connection_id| {
            debug!("{} connected: {}", id, connection_id);
            if let Some(ref subscribe_topic) = connection.subscribe_topic {
                debug!(
                    "subscribe to: {} with qos {}",
                    subscribe_topic, connection.subscribe_qos
                );
                connection
                    .client
                    .subscribe(&subscribe_topic, connection.subscribe_qos.into())
                    .unwrap();
            }
        });
        callbacks.on_disconnect(|_, connection_id| {
            debug!("{} disconnected: {}", id, connection_id);
        });
        connection.client.loop_until_disconnect(200);
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

    if let Some(ref send_topic) = connection.send_topic {
        let mut data_lock = data.lock().unwrap();
        let message_id = connection.client.publish(
            send_topic,
            serialized.as_bytes(),
            connection.send_qos.into(),
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

pub fn port_mqtt_mosquitto_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    sender_to_kernel: BoxedSender,
) {
    info!("start mqtt port with id {}", id);
    let mut connection: Option<Connection> = None;
    let mut sender: Option<Sender<ProcessingResult>> = None;
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
                match build_connection(&id, config) {
                    Ok(new_connection) => {
                        sender = Some(
                            connect(
                                id.clone(),
                                new_connection.clone(),
                                sender_to_kernel.clone_boxed(),
                                data.clone(),
                            )
                            .unwrap(),
                        );
                        connection = Some(new_connection);
                    }
                    Err(e) => error!("failed to connect {:?}", e),
                }
            }
            BrokerEvent::OutgoingCloudEvent(event) => {
                debug!("{} cloudevent received", &id);
                if let Some(ref connection) = connection {
                    debug!("{} will send event out", &id);
                    let result = send_cloud_event(&id, &event, &connection, data.clone());
                } else {
                    error!("no active connection - cant send event");
                }
            }
            BrokerEvent::IncomingCloudEventProcessed(event_id, result) => {
                // todo check result
                if let Some(ref sender) = sender {
                    debug!(
                        "received IncomingCloudEventProcessed -> send result to on_message handler"
                    );
                    sender.send(result).unwrap();
                } else {
                    error!("no active connection - cant send result");
                }
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

/// This is the pointer for the main function to start the port.
pub static PORT_MQTT_MOSQUITTO: InternalServerFnRefStatic =
    &(port_mqtt_mosquitto_start as InternalServerFn);
