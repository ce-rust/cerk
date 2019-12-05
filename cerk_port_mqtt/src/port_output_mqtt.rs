use cerk::kernel::{BrokerEvent, Config};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use paho_mqtt::{AsyncClient, ConnectOptions, CreateOptionsBuilder, Message, PersistenceType};
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

/// MQTT Port
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
                if let Some(cli) = cli.as_ref() {
                    let msg_string = format!("Hello Rust MQTT world! {}", cloud_event.event_id());
                    let msg = Message::new("test", msg_string, 0);
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
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}
