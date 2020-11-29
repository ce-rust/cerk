#[macro_use]
extern crate log;
use env_logger::Env;

use cerk::kernel::{bootstrap, BrokerEvent, Config, ScheduleInternalServer, StartOptions};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerId};
use cerk_port_dummies::PORT_PRINTER;
use cerk_port_mqtt::PORT_MQTT;
use cerk_router_broadcast::ROUTER_BROADCAST;
use cerk_runtime_threading::THREADING_SCHEDULER;
use std::collections::HashMap;
use std::env;

const MQTT_INPUT: &'static str = "mqtt-input";
const DUMMY_LOGGER_OUTPUT: &'static str = "dummy-logger-output";

fn static_config_loader_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    sender_to_kernel: BoxedSender,
) {
    info!("start static config loader with id {}", id);
    let mqtt_broker_url: String =
        env::var("MQTT_BROKER_URL").unwrap_or(String::from("tcp://mqtt-broker:1883"));
    let mqtt_in_config: HashMap<String, Config> = [
        ("host".to_string(), Config::String(mqtt_broker_url)),
        (
            "subscribe_topic".to_string(),
            Config::String("test".to_string()),
        ),
        ("subscribe_qos".to_string(), Config::U8(1)),
    ]
    .iter()
    .cloned()
    .collect();

    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                sender_to_kernel.send(BrokerEvent::ConfigUpdated(
                    Config::Vec(vec![Config::String(String::from(DUMMY_LOGGER_OUTPUT))]),
                    String::from("router"),
                ));
                sender_to_kernel.send(BrokerEvent::ConfigUpdated(
                    Config::HashMap(mqtt_in_config.clone()),
                    String::from(MQTT_INPUT),
                ));
                sender_to_kernel.send(BrokerEvent::ConfigUpdated(
                    Config::Null,
                    String::from(DUMMY_LOGGER_OUTPUT),
                ));
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

fn main() {
    env_logger::from_env(Env::default().default_filter_or("debug")).init();
    info!("start mqtt to printer router");
    let start_options = StartOptions {
        scheduler: THREADING_SCHEDULER,
        router: ROUTER_BROADCAST,
        config_loader: &(static_config_loader_start as InternalServerFn),
        ports: vec![
            ScheduleInternalServer {
                id: String::from(MQTT_INPUT),
                function: PORT_MQTT,
            },
            ScheduleInternalServer {
                id: String::from(DUMMY_LOGGER_OUTPUT),
                function: PORT_PRINTER,
            },
        ],
    };
    bootstrap(start_options);
}
