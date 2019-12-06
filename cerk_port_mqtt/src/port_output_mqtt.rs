use cerk::kernel::{BrokerEvent, Config};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use cloudevents::CloudEvent;
use paho_mqtt::{AsyncClient, ConnectOptions, CreateOptionsBuilder, Message, PersistenceType};
use serde_json;
use std::time::Duration;

fn setup_connection(
    id: &InternalServerId,
    old_cli: Option<AsyncClient>,
    config: Config,
) -> Option<AsyncClient> {
    let options = match config {
        Config::String(host) => {
            info!("new config");
            CreateOptionsBuilder::new()
                .server_uri(host)
                .persistence(PersistenceType::None)
                .finalize()
        }
        _ => panic!("{} received invalide config", id),
    };

    if let Some(cli) = old_cli {
        cli.disconnect(None);
    }
    let cli = AsyncClient::new(options).unwrap_or_else(|err| {
        panic!("Error creating the client: {}", err);
    });

    if let Err(e) = cli
        .connect(ConnectOptions::new())
        .wait_for(Duration::from_secs(1))
    {
        panic!("Unable to connect: {:?}", e);
    }

    Some(cli)
}

fn send_cloud_event(id: &InternalServerId, cloud_event: &CloudEvent, cli: &Option<AsyncClient>) {
    if let Some(cli) = cli.as_ref() {
        let serialized = serde_json::to_string(cloud_event);
        let msg = Message::new("test", serialized.unwrap(), 0);
        let tok = cli.publish(msg);

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
/// TODO
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

    info!("start mqtt port with id {}", id);

    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                info!("{} initiated", id);
            }
            BrokerEvent::ConfigUpdated(config, _) => {
                info!("{} received ConfigUpdated", &id);
                cli = setup_connection(&id, cli, config);
            }
            BrokerEvent::OutgoingCloudEvent(cloud_event, _) => {
                debug!("{} cloudevent received", &id);
                send_cloud_event(&id, &cloud_event, &cli);
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}
