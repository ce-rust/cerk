#[macro_use]
extern crate log;
use env_logger::Env;

use cerk::kernel::{bootstrap, BrokerEvent, Config, ScheduleInternalServer, StartOptions};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerId};
use cerk_port_dummies::{PORT_PRINTER, PORT_SEQUENCE_GENERATOR};
use cerk_router_rule_based::{CloudEventFields, RoutingRules, RoutingTable, ROUTER_RULE_BASED};
use cerk_runtime_threading::THREADING_SCHEDULER;
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
                Some(DUMMY_SEQUENCE_GENERATOR.to_string()),
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
        scheduler: THREADING_SCHEDULER,
        router: ROUTER_RULE_BASED,
        config_loader: &(static_config_loader_start as InternalServerFn),
        ports: vec![
            ScheduleInternalServer {
                id: String::from(DUMMY_SEQUENCE_GENERATOR),
                function: PORT_SEQUENCE_GENERATOR,
            },
            ScheduleInternalServer {
                id: String::from(DUMMY_LOGGER_OUTPUT),
                function: PORT_PRINTER,
            },
        ],
    };
    bootstrap(start_options);
}
