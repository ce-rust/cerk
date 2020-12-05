use mosquitto_client::{Mosquitto,Callbacks};
use cerk::kernel::{BrokerEvent, CloudEventRoutingArgs, Config, IncomingCloudEvent, DeliveryGuarantee, OutgoingCloudEventProcessed, RoutingResult, ProcessingResult};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerFnRefStatic, InternalServerId};
use cloudevents::{AttributesReader, Event};
use serde_json;
use std::{thread,time};
use anyhow::Result;
use std::sync::mpsc::{channel,Sender};

fn build_connection(id: &InternalServerId, config: Config) -> Result<Mosquitto> {
    match config {
        Config::HashMap(ref config_map) => {
            // let host = match config_map.get("host") {
            //     Some(Config::String(host)) => host,
            //     _ => panic!("{} invalid value for host", id),
            // };
            //
            // let send_topic = match config_map.get("send_topic") {
            //     Some(Config::String(topic)) => Some(topic.clone()),
            //     Some(_) => panic!("{} invalid value for send_topic", id),
            //     _ => None,
            // };
            //
            // let send_qos = match config_map.get("send_qos") {
            //     Some(Config::U8(qos)) => *qos,
            //     Some(_) => panic!("{} invalid value for send_qos", id),
            //     _ => 0,
            // };
            //
            // let subscribe_topic = match config_map.get("subscribe_topic") {
            //     Some(Config::String(topic)) => Some(topic.clone()),
            //     Some(_) => panic!("{} invalid value for subscribe_topic", id),
            //     _ => None,
            // };
            //
            // let subscribe_qos = match config_map.get("subscribe_qos") {
            //     Some(Config::U8(qos)) => *qos,
            //     Some(_) => panic!("{} invalid value for subscribe_qos", id),
            //     _ => 0,
            // };

            let client = mosquitto_client::Mosquitto::new_session(&id.clone(), false); // keep old session
            client.threaded();
            client.connect("localhost", 1883, 5)?;
            client.subscribe("inbox",1)?;

            return Ok(client);
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
    client: &Mosquitto,
) -> Result<()> {
    let serialized = serde_json::to_string(cloud_event)?;
    // todo wait for publish confirm
    let id = client.publish("outbox", serialized.as_bytes(), 1, false)?;
    debug!("sent publish with id {}", id);
    return Ok(());
}


pub fn port_mqtt_mosquitto_start(id: InternalServerId, inbox: BoxedReceiver, sender_to_kernel: BoxedSender) {
    info!("start mqtt port with id {}", id);
    let mut client: Option<Mosquitto> = None;
    let mut sender: Option<Sender<()>> = None;

    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                info!("{} initiated", id);
            }
            BrokerEvent::ConfigUpdated(config, _) => {
                info!("{} received ConfigUpdated", &id);
                match build_connection(&id, config) {
                    Ok(new_client) => client = Some(new_client),
                    Err(e) => error!("failed to connect {:?}", e)
                }

                if let Some(ref client) = client {
                    sender = Some(connect(id.clone(), client.clone(), sender_to_kernel.clone_boxed()).unwrap());
                } else {
                    // TODO
                }
            }
            BrokerEvent::OutgoingCloudEvent(event) => {
                debug!("{} cloudevent received", &id);
                if let Some(ref client) = client {
                    debug!("{} will send event out", &id);
                    let result = send_cloud_event(&id, &event.cloud_event, &client);
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
