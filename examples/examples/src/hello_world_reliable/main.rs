#[macro_use]
extern crate log;

use env_logger::Env;

use cerk::kernel::{
    bootstrap, BrokerEvent, Config, DeliveryGuarantee, ScheduleInternalServer, StartOptions,
};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerId};
use cerk_port_dummies::{PORT_SEQUENCE_GENERATOR, PORT_SEQUENCE_VALIDATOR};
use cerk_router_broadcast::ROUTER_BROADCAST;
use cerk_runtime_threading::THREADING_SCHEDULER;
use std::env;

const DUMMY_SEQUENCE_GENERATOR: &'static str = "dummy-sequence-generator";
const DUMMY_OUTPUT: &'static str = "dummy-validator-output";

fn static_config_loader_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    sender_to_kernel: BoxedSender,
) {
    info!("start static config loader with id {}", id);
    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                sender_to_kernel.send(BrokerEvent::ConfigUpdated(
                    Config::Vec(vec![Config::String(String::from(DUMMY_OUTPUT))]),
                    String::from("router"),
                ));
                sender_to_kernel.send(BrokerEvent::ConfigUpdated(
                    Config::HashMap(
                        [(
                            "delivery_guarantee".to_string(),
                            Config::from(DeliveryGuarantee::AtLeastOnce),
                        )]
                        .iter()
                        .cloned()
                        .collect(),
                    ),
                    String::from(DUMMY_SEQUENCE_GENERATOR),
                ));
                sender_to_kernel.send(BrokerEvent::ConfigUpdated(
                    Config::Null,
                    String::from(DUMMY_OUTPUT),
                ));
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

fn main() {
    env_logger::from_env(Env::default().default_filter_or("debug")).init();
    env::set_var("GENERATOR_AMOUNT", "50");
    env::set_var("VALIDATOR_AMOUNT", "50");
    info!("start hello world example");
    let start_options = StartOptions {
        scheduler: THREADING_SCHEDULER,
        router: ROUTER_BROADCAST,
        config_loader: &(static_config_loader_start as InternalServerFn),
        ports: vec![
            ScheduleInternalServer {
                id: String::from(DUMMY_SEQUENCE_GENERATOR),
                function: PORT_SEQUENCE_GENERATOR,
            },
            ScheduleInternalServer {
                id: String::from(DUMMY_OUTPUT),
                function: PORT_SEQUENCE_VALIDATOR,
            },
        ],
    };
    bootstrap(start_options);
}
