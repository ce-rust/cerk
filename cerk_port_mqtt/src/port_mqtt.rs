use anyhow::{bail, Result};
use async_std::task::block_on;
use cerk::kernel::{
    BrokerEvent, CloudEventRoutingArgs, Config, DeliveryGuarantee, IncomingCloudEvent,
    OutgoingCloudEvent, OutgoingCloudEventProcessed, ProcessingResult,
};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerFnRefStatic, InternalServerId};
use cloudevents::{AttributesReader, Event};
use paho_mqtt::{
    AsyncClient, ConnectOptionsBuilder, CreateOptionsBuilder, Message, PersistenceType,
};
use serde_json;
use std::time::Duration;

struct MqttConnection {
    client: AsyncClient,
    send_topic: Option<String>,
    subscribe_topic: Option<String>,
}

fn build_connection(id: &InternalServerId, config: Config) -> MqttConnection {
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

            let subscribe_topic = match config_map.get("subscribe_topic") {
                Some(Config::String(topic)) => Some(topic.clone()),
                Some(_) => panic!("{} invalid value for subscribe_topic", id),
                _ => None,
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
                send_topic,
                subscribe_topic,
            };
        }
        _ => panic!("{} received invalide config", id),
    }
}

fn message_handler(
    id: InternalServerId,
    sender_to_kernel: BoxedSender,
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
                        let routing_id = cloud_event.id().to_string();
                        sender_to_kernel.send(BrokerEvent::IncomingCloudEvent(
                            IncomingCloudEvent {
                                routing_id,
                                incoming_id: id.clone(),
                                cloud_event,
                                args: routing_args.clone(),
                            },
                        ));
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
    sender_to_kernel: BoxedSender,
    config: Config,
) -> Result<MqttConnection> {
    debug!("{} start connection to mqtt broker", id);

    let mut connection = build_connection(id, config);

    let connection_options = ConnectOptionsBuilder::new()
        .clean_session(false)
        .clean_start(false)
        .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(5))
        .finalize();

    connection.client.connect(connection_options).await?;

    let routing_args = CloudEventRoutingArgs {
        delivery_guarantee: DeliveryGuarantee::BestEffort,
    };

    connection.client.set_message_callback(message_handler(
        id.clone(),
        sender_to_kernel,
        routing_args,
    ));

    if let Some(ref subscribe_topic) = connection.subscribe_topic {
        debug!("{} subscribes to {} with qos 0", id, subscribe_topic,);

        connection.client.subscribe(subscribe_topic, 0).await?;
    }

    Ok(connection)
}

async fn send_cloud_event(
    id: &InternalServerId,
    event: &OutgoingCloudEvent,
    connection: &MqttConnection,
) -> Result<ProcessingResult> {
    if let Some(ref send_topic) = connection.send_topic {
        let serialized = serde_json::to_string(&event.cloud_event).unwrap();
        debug!("{} message serialized", id);
        let send_qos = match event.args.delivery_guarantee {
            DeliveryGuarantee::BestEffort => 0,
            DeliveryGuarantee::AtLeastOnce => 1,
        };
        let msg = Message::new(send_topic, serialized, send_qos);
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

/// This is the main function to start the port.
pub fn port_mqtt_start(id: InternalServerId, inbox: BoxedReceiver, sender_to_kernel: BoxedSender) {
    let mut connection: Option<MqttConnection> = None;

    info!("start mqtt port with id {}", id);

    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                info!("{} initiated", id);
            }
            BrokerEvent::ConfigUpdated(config, _) => {
                info!("{} received ConfigUpdated", &id);
                if let Some(ref connection) = connection {
                    match block_on(connection.client.disconnect(None)) {
                        Ok(_) => debug!("disconnected succesfully"),
                        Err(err) => panic!("{} disconnects failed {:?}", id, err),
                    }
                }

                match block_on(setup_connection(
                    &id,
                    sender_to_kernel.clone_boxed(),
                    config,
                )) {
                    Ok(new_connection) => {
                        connection = Some(new_connection);
                    }
                    Err(err) => panic!("{} connection setup failed {:?}", id, err),
                }
            }
            BrokerEvent::OutgoingCloudEvent(event) => {
                debug!("{} cloudevent received", &id);
                if let Some(ref connection) = connection {
                    match block_on(send_cloud_event(&id, &event, &connection)) {
                        Ok(result) => {
                            debug!("{} cloudevent sent -> {:?}", &id, &result);

                            sender_to_kernel.send(BrokerEvent::OutgoingCloudEventProcessed(
                                OutgoingCloudEventProcessed {
                                    sender_id: id.clone(),
                                    routing_id: event.routing_id,
                                    result: result,
                                },
                            ));
                        }
                        Err(err) => panic!("{} connection setup failed {:?}", id, err),
                    }
                } else {
                    panic!("{} can not send message, no connection configured", id)
                }
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

/// This is the pointer for the main function to start the port.
pub static PORT_MQTT: InternalServerFnRefStatic = &(port_mqtt_start as InternalServerFn);
