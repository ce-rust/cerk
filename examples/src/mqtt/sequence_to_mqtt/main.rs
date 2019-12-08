#[macro_use]
extern crate log;
use env_logger::Env;

use cerk::kernel::{bootstrap, BrokerEvent, Config, StartOptions};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use cerk_port_dummies::port_sequence_generator_start;
use cerk_port_mqtt::port_mqtt_start;
use cerk_router_broadcast::router_start;
use cerk_runtime_threading::threading_scheduler_start;
use std::collections::HashMap;

const DUMMY_SEQUENCE_GENERATOR: &'static str = "dummy-sequence-generator";
const MQTT_OUTPUT: &'static str = "mqtt-output";

fn static_config_loader_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    sender_to_kernel: BoxedSender,
) {
    info!("start static config loader with id {}", id);

    let mqtt_out_config: HashMap<String, Config> = [
        (
            "host".to_string(),
            Config::String("tcp://mqtt-broker:1883".to_string()),
        ),
        ("send_topic".to_string(), Config::String("test".to_string())),
    ]
    .iter()
    .cloned()
    .collect();

    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                sender_to_kernel.send(BrokerEvent::ConfigUpdated(
                    Config::Vec(vec![Config::String(String::from(MQTT_OUTPUT))]),
                    String::from("router"),
                ));
                sender_to_kernel.send(BrokerEvent::ConfigUpdated(
                    Config::Null,
                    String::from(DUMMY_SEQUENCE_GENERATOR),
                ));
                sender_to_kernel.send(BrokerEvent::ConfigUpdated(
                    Config::HashMap(mqtt_out_config.clone()),
                    String::from(MQTT_OUTPUT),
                ));
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

fn main() {
    env_logger::from_env(Env::default().default_filter_or("debug")).init();
    info!("start sequenc generater to mqtt router");
    let start_options = StartOptions {
        scheduler_start: threading_scheduler_start,
        router_start: router_start,
        config_loader_start: static_config_loader_start,
        ports: Box::new([
            (
                String::from(DUMMY_SEQUENCE_GENERATOR),
                port_sequence_generator_start,
            ),
            (String::from(MQTT_OUTPUT), port_mqtt_start),
        ]),
    };
    bootstrap(start_options);
}
