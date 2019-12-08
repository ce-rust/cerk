use cerk::kernel::{BrokerEvent, Config};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use cloudevents::CloudEvent;
use paho_mqtt::{
    AsyncClient, ConnectOptions, CreateOptions, CreateOptionsBuilder, Message, PersistenceType,
};
use serde_json;
use std::time::Duration;

fn build_configs(id: &InternalServerId, config: Config) -> (CreateOptions, String, u8) {
    match config {
        Config::HashMap(ref config_map) => {
            let mut mqtt_config = CreateOptionsBuilder::new();
            if let Config::String(host) = &config_map["host"] {
                info!("new config");
                mqtt_config = mqtt_config.server_uri(host);
            } else {
                panic!("{} received invalide config, no host as String", id);
            }

            let topic = if let Config::String(topic) = &config_map["topic"] {
                topic.clone()
            } else {
                panic!("{} received invalide config, no topic as String", id);
            };

            let qos = if let Some(Config::U8(qos)) = config_map.get("qos") {
                *qos
            } else {
                0
            };

            if let Some(Config::U8(persistence)) = config_map.get("persistence") {
                match persistence {
                    0 => mqtt_config = mqtt_config.persistence(PersistenceType::File),
                    1 => mqtt_config = mqtt_config.persistence(PersistenceType::None),
                    _ => panic!("{} received invalide config: persistence", id),
                }
            } else {
                mqtt_config = mqtt_config.persistence(PersistenceType::None);
            }
            (mqtt_config.finalize(), topic, qos)
        }
        _ => panic!("{} received invalide config", id),
    }
}

fn setup_connection(
    id: &InternalServerId,
    old_cli: Option<AsyncClient>,
    config: Config,
) -> (AsyncClient, String, u8) {
    let (options, topic, qos) = build_configs(id, config);

    if let Some(cli) = old_cli {
        cli.disconnect(None);
    }
    let mut cli = AsyncClient::new(options).unwrap_or_else(|err| {
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

    (cli, topic, qos)
}

fn send_cloud_event(
    id: &InternalServerId,
    cloud_event: &CloudEvent,
    cli: &Option<AsyncClient>,
    topic: &Option<String>,
    qos: u8,
) {
    if cli.is_some() && topic.is_some() {
        let serialized = serde_json::to_string(cloud_event);
        let msg = Message::new(topic.as_ref().unwrap(), serialized.unwrap(), qos as i32);
        let tok = cli.as_ref().unwrap().publish(msg);

        if let Err(e) = tok.wait_for(Duration::from_secs(1)) {
            panic!("Error sending message: {:?}", e);
        }
    } else {
        error!(
            "{} received event before it was configured -> message will be dropped",
            id
        );
    }
}

/// This port publishes CloudEvents to a MQTT v3.1 topic.
///
/// The port is implemented with a [Eclipse Paho MQTT Rust Client](https://github.com/eclipse/paho.mqtt.rust)
/// and sends messages according to the
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
/// ## topic
///
/// The value has to by of type `Config::String` and contain the MQTT topic name.
///
/// E.g. `Config::String(String::from("test"))`
///
/// ## Optional Fields
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
/// ### qos
///
/// The [quality of service](http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718099) for message delivery.
/// The quality of service is only for the MQTT broker and does not change any behavior of the router or the output port.
/// The router only supports best effort at the moment.  
///
/// * 0: At most once delivery (default)
/// * 1: At least once delivery
/// * 2: Exactly once delivery
///
/// ## Configuration Examples
///
/// ### Minimal Configuration
///
/// ```
/// use std::collections::HashMap;
/// use cerk::kernel::Config;
///
/// let map: HashMap<String, Config> = [
///     ("host".to_string(), Config::String("tcp://mqtt-broker:1883".to_string())),
///     ("topic".to_string(), Config::String("test".to_string())),
/// ]
/// .iter()
/// .cloned()
/// .collect();
///
/// let config = Config::HashMap(map);
/// ```
///
/// ### Full Configuration
///
/// ```
/// use std::collections::HashMap;
/// use cerk::kernel::Config;
///
/// let map: HashMap<String, Config> = [
///     ("host".to_string(), Config::String("tcp://mqtt-broker:1883".to_string())),
///     ("topic".to_string(), Config::String("test".to_string())),
///     ("persistance".to_string(), Config::U8(0)),
///     ("qos".to_string(), Config::U8(2)),
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
/// * [Generator to MQTT](https://github.com/ce-rust/cerk/tree/master/examples/src/sequence_to_mqtt/)
///
pub fn port_output_mqtt_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    _sender_to_kernel: BoxedSender,
) {
    let mut cli: Option<AsyncClient> = None;
    let mut topic: Option<String> = None;
    let mut qos: u8 = 0;

    info!("start mqtt port with id {}", id);

    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                info!("{} initiated", id);
            }
            BrokerEvent::ConfigUpdated(config, _) => {
                info!("{} received ConfigUpdated", &id);
                let (new_cli, new_topic, new_qos) = setup_connection(&id, cli, config);
                cli = Some(new_cli);
                topic = Some(new_topic);
                qos = new_qos;
            }
            BrokerEvent::OutgoingCloudEvent(cloud_event, _) => {
                debug!("{} cloudevent received", &id);
                send_cloud_event(&id, &cloud_event, &cli, &topic, qos);
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
    fn setup_connection_with_minimal_config() {
        let map: HashMap<String, Config> = [
            (
                "host".to_string(),
                Config::String("tcp://mqtt-broker:1883".to_string()),
            ),
            ("topic".to_string(), Config::String("test".to_string())),
        ]
        .iter()
        .cloned()
        .collect();

        let config = Config::HashMap(map);

        let (_, topic, qos) = build_configs(&"test".to_string(), config);
        assert_eq!(topic, "test".to_string());
        assert_eq!(qos, 0);
    }
    #[test]
    fn setup_connection_with_full_config() {
        let map: HashMap<String, Config> = [
            (
                "host".to_string(),
                Config::String("tcp://mqtt-broker:1883".to_string()),
            ),
            ("topic".to_string(), Config::String("test".to_string())),
            ("persistance".to_string(), Config::U8(0)),
            ("qos".to_string(), Config::U8(2)),
        ]
        .iter()
        .cloned()
        .collect();

        let config = Config::HashMap(map);

        let (_, topic, qos) = build_configs(&"test".to_string(), config);
        assert_eq!(topic, "test".to_string());
        assert_eq!(qos, 2);
    }
}
