use cerk::kernel::{BrokerEvent, Config, CloudEventRoutingArgs};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use cloudevents::CloudEvent;
use paho_mqtt::{
    AsyncClient, ConnectOptions, CreateOptions, CreateOptionsBuilder, Message, PersistenceType,
};
use serde_json;
use std::rc::Rc;
use std::time::Duration;

struct MqttOptions {
    send_topic: Option<String>,
    send_qos: u8,
    subscribe_topics: Vec<String>,
    subscribe_qos: Vec<u8>,
}

fn build_configs(id: &InternalServerId, config: Config) -> (CreateOptions, MqttOptions) {
    match config {
        Config::HashMap(ref config_map) => {
            let mut mqtt_config = CreateOptionsBuilder::new().client_id(format!("cerk-{}", id));
            if let Config::String(host) = &config_map["host"] {
                info!("new config");
                mqtt_config = mqtt_config.server_uri(host);
            } else {
                panic!("{} received invalide config, no host as String", id);
            }

            let send_topic = if let Some(Config::String(topic)) = config_map.get("send_topic") {
                Some(topic.clone())
            } else {
                None
            };

            let send_qos = if let Some(Config::U8(qos)) = config_map.get("send_qos") {
                *qos
            } else {
                0
            };

            let subscribe_topics = if let Some(Config::Vec(topics)) =
                config_map.get("subscribe_topics")
            {
                topics.into_iter().map(|item|{
                        if let Config::String(item) = item {
                            item.clone()
                        }else{
                            panic!("{} received invalide config, subscribe_topics is not Config::Vec of Strings", id);
                        }
                    }).collect()
            } else {
                vec![]
            };

            let subscribe_qos = if let Some(Config::Vec(qos)) = config_map.get("subscribe_qos") {
                qos.into_iter().map(|item|{
                    if let Config::U8(item) = item {
                        *item
                    }else{
                        panic!("{} received invalide config, subscribe_qos is not Config::Vec of U8s", id);
                    }
                }).collect()
            } else {
                vec![]
            };

            if subscribe_topics.len() != subscribe_qos.len() {
                panic!("{} received invalide config: subscribe_topics and subscribe_qos needs to have the same size", id);
            }

            if let Some(Config::U8(persistence)) = config_map.get("persistence") {
                match persistence {
                    0 => mqtt_config = mqtt_config.persistence(PersistenceType::File),
                    1 => mqtt_config = mqtt_config.persistence(PersistenceType::None),
                    _ => panic!("{} received invalide config: persistence", id),
                }
            } else {
                mqtt_config = mqtt_config.persistence(PersistenceType::None);
            }
            (
                mqtt_config.finalize(),
                MqttOptions {
                    send_topic,
                    send_qos,
                    subscribe_topics,
                    subscribe_qos,
                },
            )
        }
        _ => panic!("{} received invalide config", id),
    }
}

fn setup_connection(
    id: &InternalServerId,
    sender_to_kernel: &BoxedSender,
    old_cli: Option<AsyncClient>,
    config: Config,
) -> (AsyncClient, MqttOptions) {
    let (crate_configs, options) = build_configs(id, config);

    if let Some(cli) = old_cli {
        cli.disconnect(None);
    }

    debug!("{} start connection to mqtt broker", id);

    let mut cli = AsyncClient::new(crate_configs).unwrap_or_else(|err| {
        panic!("Error creating the client: {}", err);
    });

    if let Err(e) = cli
        .connect(ConnectOptions::new())
        .wait_for(Duration::from_secs(1))
    {
        panic!("Unable to connect: {:?}", e);
    }

    cli.set_connection_lost_callback(|cli: &AsyncClient| {
        warn!("Connection lost. Attempting reconnect.");
        let tok = cli.reconnect();
        if let Err(e) = tok.wait_for(Duration::from_secs(1)) {
            panic!("Unable to reconnect: {:?}", e);
        }
    });

    let rc_id = Rc::new(id.clone());
    let rc_send = Rc::new((*sender_to_kernel).clone_boxed());
    cli.set_message_callback(move |_cli, msg| {
        debug!("{} received message callback", rc_id);
        if let Some(msg) = msg {
            let topic = msg.topic();
            let payload_str = msg.payload_str();
            debug!("{} received cloudevent on topic {}", rc_id, topic);
            match serde_json::from_str::<CloudEvent>(&payload_str) {
                Ok(cloud_event) => {
                    debug!("{} deserialized event successfully", rc_id);
                    rc_send.send(BrokerEvent::IncomingCloudEvent(
                        (*rc_id).clone(),
                        cloud_event,
                        CloudEventRoutingArgs::default(), // todo correct args
                    ))
                }
                Err(err) => {
                    error!("{} while converting string to CloudEvent: {:?}", rc_id, err);
                }
            }
        }
    });

    if options.subscribe_topics.len() > 0 {
        debug!(
            "{} subscribes to {:?} with qos {:?}",
            id, options.subscribe_topics, options.subscribe_qos,
        );
        let topics = options
            .subscribe_topics
            .iter()
            .map(|s| &**s)
            .collect::<Vec<&str>>();

        // has not worked with subscribe_many
        for i in 0..topics.len() {
            let tok = cli.subscribe(topics[i], options.subscribe_qos[i] as i32);
            if let Err(e) = tok.wait_for(Duration::from_secs(1)) {
                panic!("Error sending message: {:?}", e);
            }
        }
    }

    (cli, options)
}

fn send_cloud_event(
    id: &InternalServerId,
    cloud_event: &CloudEvent,
    cli: &Option<AsyncClient>,
    options: &Option<MqttOptions>,
) {
    if cli.is_some() && options.is_some() && options.as_ref().unwrap().send_topic.is_some() {
        let serialized = serde_json::to_string(cloud_event);
        let msg = Message::new(
            options.as_ref().unwrap().send_topic.as_ref().unwrap(),
            serialized.unwrap(),
            options.as_ref().unwrap().send_qos as i32,
        );
        let tok = cli.as_ref().unwrap().publish(msg);

        if let Err(e) = tok.wait_for(Duration::from_secs(1)) {
            panic!("Error sending message: {:?}", e);
        }
    } else {
        error!(
            "{} received event before the mqtt port was configured as output port -> message will be dropped",
            id
        );
    }
}

/// This port publishes and/or subscribe CloudEvents to/from an MQTT v3.1 topic.
///
/// The port is implemented with a [Eclipse Paho MQTT Rust Client](https://github.com/eclipse/paho.mqtt.rust)
/// and sends and receives messages according to the
/// [MQTT Protocol Binding for CloudEvents v1.0](https://github.com/cloudevents/spec/blob/v1.0/mqtt-protocol-binding.md)
/// specification
///
/// # Configurations
///
/// The configurations should be of type `cerk::kernel::Config::HashMap` and have at least the entires:
///
/// ## Required Fields
///
/// ## host
///
/// The value has to by of type `Config::String` and contain a host name with protocol and port.
///
/// E.g. `Config::String(String::from("tcp://mqtt-broker:1883"))`
///
/// ## Optional Fields
///
/// ### send_topic
///
/// The value has to by of type `Config::String` and contain the MQTT topic name where the message will be sent to.
///
/// E.g. `Config::String(String::from("test"))`
///
/// The following configurations are optional.
///
/// ### persistance
///
/// The value has to by of type `Config::U8` and contain one of the following values.
///
/// The values are defined according to the Eclipse Paho MQTT Rust Client PersistenceType.
///
/// * 0: File (default) -  Data and messages are persisted to a local file (default)
/// * 1: None - No persistence is used.
///
/// E.g. `Config::U8(0)`
///
/// ### send_qos
///
/// The [quality of service](http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718099) for message delivery.
/// The quality of service is only for the MQTT broker and does not change any behavior of the router or the port.
/// The router only supports best effort at the moment.  
///
/// * 0: At most once delivery (default)
/// * 1: At least once delivery
/// * 2: Exactly once delivery
///
/// E.g. `Config::U8(0)`
///
/// ## subscribe_topics
///
/// The value has to by of type `Config::Vec([Config::String])` and must have the same length as `subscribe_qos`.
/// The values in the vector contain the MQTT topic wich the router should subscribe to.
///
/// If multiple topics are subscribed in the same MQTT port,
/// there is no possability at the moment to know let the router or the output port know from wich topic the an event was received.
///
/// ## subscribe_qos
///
/// The value has to by of type `Config::Vec([Config::U8])` and must have the same length as `subscribe_topics`.
///
/// The [quality of service](http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718099) for the topic subscription.
/// The quality of service is only for the MQTT broker and does not change any behavior of the router or the port.
/// The router only supports best effort at the moment.  
///
/// * 0: At most once delivery
/// * 1: At least once delivery
/// * 2: Exactly once delivery
///
/// ## Configuration Examples
///
/// ### Minimal Configuration to send events
///
/// This configuration will connect to the borker but nor send or receive events.
///
/// ```
/// use std::collections::HashMap;
/// use cerk::kernel::Config;
///
/// let map: HashMap<String, Config> = [
///     ("host".to_string(), Config::String("tcp://mqtt-broker:1883".to_string())),
/// ]
/// .iter()
/// .cloned()
/// .collect();
///
/// let config = Config::HashMap(map);
/// ```
///
/// ### Full Configuration for sending events
///
/// ```
/// use std::collections::HashMap;
/// use cerk::kernel::Config;
///
/// let map: HashMap<String, Config> = [
///     ("host".to_string(), Config::String("tcp://mqtt-broker:1883".to_string())),
///     ("persistance".to_string(), Config::U8(0)),
///     ("send_topic".to_string(), Config::String("test".to_string())),
///     ("send_qos".to_string(), Config::U8(2)),
/// ]
/// .iter()
/// .cloned()
/// .collect();
///
/// let config = Config::HashMap(map);
/// ```
///
/// ### Full Configuration for recieve events
///
/// ```
/// use std::collections::HashMap;
/// use cerk::kernel::Config;
///
/// let map: HashMap<String, Config> = [
///     ("host".to_string(), Config::String("tcp://mqtt-broker:1883".to_string())),
///     ("persistance".to_string(), Config::U8(0)),
///     (
///       "subscribe_topics".to_string(),
///       Config::Vec(vec![Config::String("test".to_string())]),
///     ),
///     (
///       "subscribe_qos".to_string(),
///       Config::Vec(vec![Config::U8(2)]),
///     ),
/// ]
/// .iter()
/// .cloned()
/// .collect();
///
/// let config = Config::HashMap(map);
/// ```
///
/// ### Full Configuration for receiving events
///
/// ```
/// use std::collections::HashMap;
/// use cerk::kernel::Config;
///
/// let map: HashMap<String, Config> = [
///     ("host".to_string(), Config::String("tcp://mqtt-broker:1883".to_string())),
///     ("persistance".to_string(), Config::U8(0)),
///     ("send_topic".to_string(), Config::String("test".to_string())),
///     ("send_qos".to_string(), Config::U8(2)),
///     (
///       "subscribe_topics".to_string(),
///       Config::Vec(vec![Config::String("test".to_string())]),
///     ),
///     (
///       "subscribe_qos".to_string(),
///       Config::Vec(vec![Config::U8(2)]),
///     ),
/// ]
/// .iter()
/// .cloned()
/// .collect();
///
/// let config = Config::HashMap(map);
/// ```
///
/// # Examples
///
/// * [Generator to MQTT](https://github.com/ce-rust/cerk/tree/master/examples/src/mqtt/)
///
pub fn port_mqtt_start(id: InternalServerId, inbox: BoxedReceiver, sender_to_kernel: BoxedSender) {
    let mut cli: Option<AsyncClient> = None;
    let mut options: Option<MqttOptions> = None;

    info!("start mqtt port with id {}", id);

    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                info!("{} initiated", id);
            }
            BrokerEvent::ConfigUpdated(config, _) => {
                info!("{} received ConfigUpdated", &id);
                let (new_cli, new_options) = setup_connection(&id, &sender_to_kernel, cli, config);
                cli = Some(new_cli);
                options = Some(new_options);
            }
            BrokerEvent::OutgoingCloudEvent(cloud_event, _) => {
                debug!("{} cloudevent received", &id);
                send_cloud_event(&id, &cloud_event, &cli, &options);
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

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
            ("persistance".to_string(), Config::U8(0)),
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
