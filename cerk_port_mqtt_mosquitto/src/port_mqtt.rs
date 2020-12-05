use mosquitto_client::{Mosquitto,Callbacks};
use cerk::kernel::{BrokerEvent, CloudEventRoutingArgs, Config, IncomingCloudEvent};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerFnRefStatic, InternalServerId};
use cloudevents::{AttributesReader, Event};
use serde_json;
use std::{thread,time};
use anyhow::Result;

struct MosquittoClient<'a> {
    client: Option<Mosquitto>,
    callbacks: Option<Callbacks<'a, Vec<()>>>,
}

fn build_connection(id: &InternalServerId, config: Config, data: &mut MosquittoClient) -> Result<()> {
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

            data.client = Some(mosquitto_client::Mosquitto::new(&id.clone()));
            data.client.unwrap().connect("localhost", 1883)?;
            data.client.unwrap().subscribe("test",1)?;


            data.callbacks = Some(data.client.as_ref().unwrap().callbacks(Vec::<()>::new()));

            data.callbacks.unwrap().on_message(|data,msg| {
                debug!("{:?} {:?}", data, msg);
            });

            let cloned_client = data.client.unwrap().clone();
            thread::spawn(move || {
                cloned_client.loop_until_disconnect(200);
            });

            return Ok(());
        }
        _ => panic!("{} received invalide config", id),
    }
}


pub fn port_mqtt_start(id: InternalServerId, inbox: BoxedReceiver, sender_to_kernel: BoxedSender) {
    info!("start mqtt port with id {}", id);
    let mut client: MosquittoClient = MosquittoClient{
        client: None,
        callbacks: None,
    };

    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                info!("{} initiated", id);
            }
            BrokerEvent::ConfigUpdated(config, _) => {
                info!("{} received ConfigUpdated", &id);
                build_connection(&id, config, &mut client);
            }
            BrokerEvent::OutgoingCloudEvent(event) => {
                debug!("{} cloudevent received", &id);
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

/// This is the pointer for the main function to start the port.
pub static PORT_MQTT: InternalServerFnRefStatic = &(port_mqtt_start as InternalServerFn);