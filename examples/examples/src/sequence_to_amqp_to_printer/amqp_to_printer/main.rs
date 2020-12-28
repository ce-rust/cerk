#[macro_use]
extern crate log;

use env_logger::Env;

use cerk::kernel::{bootstrap, BrokerEvent, Config, ScheduleInternalServer, StartOptions};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerId};
use cerk_port_amqp::PORT_AMQP;
use cerk_port_dummies::PORT_PRINTER;
use cerk_router_broadcast::ROUTER_BROADCAST;
use cerk_runtime_threading::THREADING_SCHEDULER;
use std::collections::HashMap;
use std::env;

const AMQP_INPUT: &'static str = "ampq-input";
const DUMMY_LOGGER_OUTPUT: &'static str = "dummy-logger-output";

fn static_config_loader_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    sender_to_kernel: BoxedSender,
) {
    info!("start static config loader with id {}", id);
    let amqp_broker_uri: String =
        env::var("AMQP_BROKER_URL").unwrap_or(String::from("amqp://127.0.0.1:5672/%2f"));
    let amqp_config: HashMap<String, Config> = [
        ("uri".to_string(), Config::String(amqp_broker_uri)),
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
                    Config::HashMap(amqp_config.clone()),
                    String::from(AMQP_INPUT),
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
    info!("start amqp to printer router");
    let start_options = StartOptions {
        scheduler: THREADING_SCHEDULER,
        router: ROUTER_BROADCAST,
        config_loader: &(static_config_loader_start as InternalServerFn),
        ports: vec![
            ScheduleInternalServer {
                id: String::from(AMQP_INPUT),
                function: PORT_AMQP,
            },
            ScheduleInternalServer {
                id: String::from(DUMMY_LOGGER_OUTPUT),
                function: PORT_PRINTER,
            },
        ],
    };
    bootstrap(start_options);
}
