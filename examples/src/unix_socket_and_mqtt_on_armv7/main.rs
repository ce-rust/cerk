#[macro_use]
extern crate log;

use env_logger::Env;
use std::env;

use cerk::kernel::{bootstrap, BrokerEvent, Config, ScheduleInternalServer, StartOptions};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerId};
use cerk_port_mqtt::PORT_MQTT;
use cerk_port_unix_socket::{PORT_INPUT_UNIX_SOCKET, PORT_OUTPUT_UNIX_SOCKET};
use cerk_router_rule_based::{CloudEventFields, RoutingRules, RoutingTable, ROUTER_RULE_BASED};
use cerk_runtime_threading::THREADING_SCHEDULER;
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
        ("send_topic".to_string(), Config::String("test".to_string())),
        ("persistence".to_string(), Config::U8(1)),
    ]
    .iter()
    .cloned()
    .collect();

    let routing_rules: RoutingTable = [
        (
            PORT_UNIX_OUTPUT.to_string(),
            RoutingRules::Or(vec![
                RoutingRules::Exact(CloudEventFields::Type, Some("event.demo.A".to_string())),
                RoutingRules::Exact(CloudEventFields::Type, Some("event.demo.B".to_string())),
            ]),
        ),
        (
            PORT_MQTT_OUTPUT.to_string(),
            RoutingRules::Exact(CloudEventFields::Type, Some("event.demo.A".to_string())),
        ),
    ]
    .iter()
    .cloned()
    .collect();

    let routing_configs = serde_json::to_string(&routing_rules).unwrap();

    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                sender_to_kernel.send(BrokerEvent::Batch(vec![
                    BrokerEvent::ConfigUpdated(
                        Config::HashMap(mqtt_out_config.clone()),
                        String::from(PORT_MQTT_OUTPUT),
                    ),
                    BrokerEvent::ConfigUpdated(
                        Config::String(routing_configs.clone()),
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
        scheduler: THREADING_SCHEDULER,
        router: ROUTER_RULE_BASED,
        config_loader: &(static_config_loader_start as InternalServerFn),
        ports: vec![
            ScheduleInternalServer {
                id: String::from(PORT_UNIX_INPUT),
                function: PORT_INPUT_UNIX_SOCKET,
            },
            ScheduleInternalServer {
                id: String::from(PORT_UNIX_OUTPUT),
                function: PORT_OUTPUT_UNIX_SOCKET,
            },
            ScheduleInternalServer {
                id: String::from(PORT_MQTT_OUTPUT),
                function: PORT_MQTT,
            },
        ],
    };
    bootstrap(start_options);
}
