#[macro_use]
extern crate log;
use env_logger::Env;

use cerk::kernel::{bootstrap, BrokerEvent, Config, StartOptions};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use cerk_port_dummies::{port_printer_start, port_sequence_generator_start};
use cerk_router_rule_based::{router_start, CloudEventFields, RoutingRules, RoutingTable};
use cerk_runtime_threading::threading_scheduler_start;
use serde_json;

const DUMMY_SEQUENCE_GENERATOR: &'static str = "dummy-sequence-generator";
const DUMMY_LOGGER_OUTPUT: &'static str = "dummy-logger-output";

fn static_config_loader_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    sender_to_kernel: BoxedSender,
) {
    info!("start static config loader with id {}", id);

    let routing_rules: RoutingTable = [(
        DUMMY_LOGGER_OUTPUT.to_string(),
        RoutingRules::And(vec![
            RoutingRules::Exact(
                CloudEventFields::Source,
                Some("dummy.sequence-generator".to_string()),
            ),
            RoutingRules::EndsWith(CloudEventFields::Id, "0".to_string()),
        ]),
    )]
    .iter()
    .cloned()
    .collect();

    let routing_configs = serde_json::to_string(&routing_rules).unwrap();

    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                sender_to_kernel.send(BrokerEvent::ConfigUpdated(
                    Config::String(routing_configs.clone()),
                    String::from("router"),
                ));
                sender_to_kernel.send(BrokerEvent::ConfigUpdated(
                    Config::Null,
                    String::from(DUMMY_SEQUENCE_GENERATOR),
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
    info!("start routing example");
    let start_options = StartOptions {
        scheduler_start: threading_scheduler_start,
        router_start: router_start,
        config_loader_start: static_config_loader_start,
        ports: Box::new([
            (
                String::from(DUMMY_SEQUENCE_GENERATOR),
                port_sequence_generator_start,
            ),
            (String::from(DUMMY_LOGGER_OUTPUT), port_printer_start),
        ]),
    };
    bootstrap(start_options);
}
