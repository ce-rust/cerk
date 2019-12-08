#[macro_use]
extern crate log;
use env_logger::Env;
use std::env;

use cerk::kernel::{bootstrap, BrokerEvent, Config, StartOptions};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use cerk_port_mqtt::port_output_mqtt_start;
use cerk_port_unix_socket::{
    port_input_unix_socket_json_start, port_output_unix_socket_json_start,
};
use cerk_router_broadcast::router_start;
use cerk_runtime_threading::threading_scheduler_start;
use std::collections::HashMap;
use std::fs::remove_file;

const PORT_UNIX_INPUT: &str = "unix-json-input";
const PORT_UNIX_OUTPUT: &str = "unix-json-output";
const PORT_MQTT_OUTPUT: &str = "mqtt-output";

const SOCKET_PATH_IN: &str = "./cloud-events-in";
const SOCKET_PATH_OUT: &str = "./cloud-events-out";

fn static_config_loader_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    sender_to_kernel: BoxedSender,
) {
    info!("start static config loader with id {}", id);
    let mqtt_broker_url: String =
        env::var("MQTT_BROKER_URL").unwrap_or(String::from("tcp://localhost:1883"));
    let mqtt_out_config: HashMap<String, Config> = [
        ("host".to_string(), Config::String(mqtt_broker_url)),
        ("topic".to_string(), Config::String("test".to_string())),
    ]
    .iter()
    .cloned()
    .collect();

    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                sender_to_kernel.send(BrokerEvent::Batch(vec![
                    BrokerEvent::ConfigUpdated(
                        Config::HashMap(mqtt_out_config.clone()),
                        String::from(PORT_MQTT_OUTPUT),
                    ),
                    BrokerEvent::ConfigUpdated(
                        Config::Vec(vec![
                            Config::String(String::from(PORT_UNIX_OUTPUT)),
                            Config::String(String::from(PORT_MQTT_OUTPUT)),
                        ]),
                        String::from("router"),
                    ),
                    BrokerEvent::ConfigUpdated(
                        Config::String(String::from(SOCKET_PATH_IN)),
                        String::from(PORT_UNIX_INPUT),
                    ),
                    BrokerEvent::ConfigUpdated(
                        Config::String(String::from(SOCKET_PATH_OUT)),
                        String::from(PORT_UNIX_OUTPUT),
                    ),
                ]));
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

fn main() {
    env_logger::from_env(Env::default().default_filter_or("debug")).init();
    let _ = remove_file(SOCKET_PATH_IN);
    let _ = remove_file(SOCKET_PATH_OUT);

    info!("start UNIX Socket and MQTT example");
    let start_options = StartOptions {
        scheduler_start: threading_scheduler_start,
        router_start: router_start,
        config_loader_start: static_config_loader_start,
        ports: Box::new([
            (
                String::from(PORT_UNIX_INPUT),
                port_input_unix_socket_json_start,
            ),
            (
                String::from(PORT_UNIX_OUTPUT),
                port_output_unix_socket_json_start,
            ),
            (String::from(PORT_MQTT_OUTPUT), port_output_mqtt_start),
        ]),
    };
    bootstrap(start_options);
}
