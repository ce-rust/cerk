use mosquitto_client::{Mosquitto,Callbacks};
use cerk::kernel::{BrokerEvent, CloudEventRoutingArgs, Config, IncomingCloudEvent};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerFnRefStatic, InternalServerId};
use cloudevents::{AttributesReader, Event};
use serde_json;
use std::{thread,time};
use anyhow::Result;

fn build_connection(id: &InternalServerId, config: Config) -> Result<Mosquitto> {
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

            let client = mosquitto_client::Mosquitto::new(&id.clone());
            client.connect("localhost", 1883)?;
            client.subscribe("inbox",1)?;

            return Ok(client);
        }
        _ => panic!("{} received invalide config", id),
    }
}

fn connect(client: Mosquitto) -> Result<()> {
    thread::spawn(move || {
        let mut callbacks = client.callbacks(Vec::<()>::new());
        callbacks.on_message(|data,msg| {
            debug!("{:?} {:?}", data, msg);
        });
        client.loop_until_disconnect(200);
    });

    return Ok(());
}

fn send_cloud_event(
    id: &InternalServerId,
    cloud_event: &Event,
    client: &Mosquitto,
) -> Result<()> {
    let serialized = serde_json::to_string(cloud_event)?;
    client.publish("outbox", serialized.as_bytes(), 1, false);
    return Ok(());
}


pub fn port_mqtt_start(id: InternalServerId, inbox: BoxedReceiver, sender_to_kernel: BoxedSender) {
    info!("start mqtt port with id {}", id);
    let mut client: Option<Mosquitto> = None;

    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                info!("{} initiated", id);
            }
            BrokerEvent::ConfigUpdated(config, _) => {
                info!("{} received ConfigUpdated", &id);
                client = Some(build_connection(&id, config).unwrap());
                if let Some(ref client) = client {
                    connect(client.clone());
                }
            }
            BrokerEvent::OutgoingCloudEvent(event) => {
                debug!("{} cloudevent received", &id);
                if let Some(ref client) = client {
                    send_cloud_event(&id, &event.cloud_event, &client).unwrap();
                }
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

/// This is the pointer for the main function to start the port.
pub static PORT_MQTT: InternalServerFnRefStatic = &(port_mqtt_start as InternalServerFn);