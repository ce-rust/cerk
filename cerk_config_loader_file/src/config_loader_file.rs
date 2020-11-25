use super::file_reader::read_file;
use crate::config_parser::parse_json_to_events;
use anyhow::Result;
use cerk::kernel::BrokerEvent;
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use std::env;

pub fn read_configs_from_file(config_path: &str) -> Result<Vec<BrokerEvent>> {
    let content = read_file(config_path)?;
    parse_json_to_events(content)
}

/// This is the main function to start the config loader.
pub fn config_loader_file_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    sender_to_kernel: BoxedSender,
) {
    let config_path = env::var("CONFIG_PATH").unwrap_or(String::from("./config.json"));
    info!(
        "start file based config loader with id {}, will consume config from {}",
        id, config_path
    );
    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                info!("{} initiated", id);
                match read_configs_from_file(config_path.as_str()) {
                    Ok(config_events) => {
                        for events in config_events {
                            sender_to_kernel.send(events);
                        }
                    }
                    Err(e) => error!("failed to read config {:?}", e),
                }
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cerk::kernel::Config;
    use std::collections::HashMap;

    #[test]
    fn read_configs_from_file_sample() -> Result<()> {
        let config = read_configs_from_file("./src/test_data/amqp_to_printer.json")?;

        let amqp_config: HashMap<String, Config> = [
            (
                "uri".to_string(),
                Config::String("amqp://127.0.0.1:5672/%2f".to_string()),
            ),
            (
                "consume_channels".to_string(),
                Config::Vec(vec![Config::HashMap(
                    [
                        ("name".to_string(), Config::String("test".to_string())),
                        ("ensure_queue".to_string(), Config::Bool(true)),
                        (
                            "bind_to_exchange".to_string(),
                            Config::String("test".to_string()),
                        ),
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                )]),
            ),
            (
                "publish_channels".to_string(),
                Config::Vec(vec![Config::HashMap(
                    [
                        ("name".to_string(), Config::String("test".to_string())),
                        ("ensure_exchange".to_string(), Config::Bool(true)),
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                )]),
            ),
        ]
        .iter()
        .cloned()
        .collect();

        let vec = vec![
            BrokerEvent::ConfigUpdated(
                Config::Vec(vec![Config::String(String::from("dummy-logger-output"))]),
                String::from("router"),
            ),
            BrokerEvent::ConfigUpdated(
                Config::HashMap(amqp_config.clone()),
                String::from("ampq-input"),
            ),
            BrokerEvent::ConfigUpdated(Config::Null, String::from("dummy-logger-output")),
        ];

        let map_to_inner_tuple = |e: &BrokerEvent| {
            if let BrokerEvent::ConfigUpdated(config, service_id) = e {
                (service_id.to_string(), config.clone())
            } else {
                panic!("got BrokerEvent which is not of type ConfigUpdated");
            }
        };

        let config: HashMap<InternalServerId, Config> =
            config.iter().map(map_to_inner_tuple).collect();

        let vec: HashMap<InternalServerId, Config> = vec.iter().map(map_to_inner_tuple).collect();

        assert_eq!(config, vec);
        Ok(())
    }
}
