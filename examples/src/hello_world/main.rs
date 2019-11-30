#[macro_use]
extern crate log;
use env_logger::Env;

use cerk::kernel::{bootstrap, BrokerEvent, Config, StartOptions};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use cerk_port_dummies::{port_printer_start, port_sequence_generator_start};
use cerk_router_broadcast::router_start;
use cerk_runtime_threading::ThreadingScheduler;

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
                    Config::Array(vec![Config::String(String::from("dummy-logger-output"))]),
                    String::from("router"),
                ));
                sender_to_kernel.send(BrokerEvent::ConfigUpdated(
                    Config::Null,
                    String::from("dummy-sequence-generator"),
                ));
                sender_to_kernel.send(BrokerEvent::ConfigUpdated(
                    Config::Null,
                    String::from("dummy-logger-output"),
                ));
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

fn main() {
    env_logger::from_env(Env::default().default_filter_or("debug")).init();
    info!("start hello world example");
    let start_options = StartOptions {
        scheduler_start: ThreadingScheduler::start,
        router_start: router_start,
        config_loader_start: static_config_loader_start,
        ports: Box::new([
            (
                String::from("dummy-sequence-generator"),
                port_sequence_generator_start,
            ),
            (String::from("dummy-logger-output"), port_printer_start),
        ]),
    };
    bootstrap(start_options);
}
