#[macro_use]
extern crate log;
use env_logger::Env;

use cerk::kernel::{bootstrap, BrokerEvent, Config, ScheduleInternalServer, StartOptions};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerId};
use cerk_port_unix_socket::{PORT_INPUT_UNIX_SOCKET, PORT_OUTPUT_UNIX_SOCKET};
use cerk_router_broadcast::ROUTER_BROADCAST;
use cerk_runtime_threading::THREADING_SCHEDULER;
use std::fs::remove_file;

const PORT_UNIX_INPUT: &str = "unix-json-input";
const PORT_UNIX_OUTPUT: &str = "unix-json-output";

const SOCKET_PATH_IN: &str = "./cloud-events-in";
const SOCKET_PATH_OUT: &str = "./cloud-events-out";

fn static_config_loader_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    sender_to_kernel: BoxedSender,
) {
    info!("start static config loader with id {}", id);
    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                sender_to_kernel.send(BrokerEvent::Batch(vec![
                    BrokerEvent::ConfigUpdated(
                        Config::Vec(vec![Config::String(String::from(PORT_UNIX_OUTPUT))]),
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

    info!("start UNIX Socket example");
    let start_options = StartOptions {
        scheduler: THREADING_SCHEDULER,
        router: ROUTER_BROADCAST,
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
        ],
    };
    bootstrap(start_options);
}
