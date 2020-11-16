#[macro_use]
extern crate log;

use env_logger::Env;

use cerk::kernel::{bootstrap, BrokerEvent, Config, StartOptions};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use cerk_port_dummies::port_printer_start;
use cerk_port_amqp::port_amqp_start;
use cerk_router_broadcast::router_start;
use cerk_runtime_threading::threading_scheduler_start;
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
            Config::Vec(vec![Config::HashMap([
                ("name".to_string(), Config::String("test".to_string())),
                ("ensure_queue".to_string(), Config::Bool(true)),
            ].iter().cloned().collect())]),
        ),
        (
            "publish_channels".to_string(),
            Config::Vec(vec![Config::HashMap([
                ("name".to_string(), Config::String("test".to_string())),
                ("ensure_exchange".to_string(), Config::Bool(true)),
            ].iter().cloned().collect())]),
        )
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
    info!("start mqtt to printer router");
    let start_options = StartOptions {
        scheduler_start: threading_scheduler_start,
        router_start: router_start,
        config_loader_start: static_config_loader_start,
        ports: Box::new([
            (String::from(AMQP_INPUT), port_amqp_start),
            (String::from(DUMMY_LOGGER_OUTPUT), port_printer_start),
        ]),
    };
    bootstrap(start_options);
}
