#[macro_use]
extern crate log;

use env_logger::Env;

use cerk::kernel::{bootstrap, ScheduleInternalServer, StartOptions};
use cerk_config_loader_file::CONFIG_LOADER_FILE;
use cerk_port_amqp::PORT_AMQP;
use cerk_port_dummies::PORT_PRINTER;
use cerk_router_broadcast::ROUTER_BROADCAST;
use cerk_runtime_threading::THREADING_SCHEDULER;

const AMQP_INPUT: &'static str = "ampq-input";
const DUMMY_LOGGER_OUTPUT: &'static str = "dummy-logger-output";

fn main() {
    env_logger::from_env(Env::default().default_filter_or("debug")).init();
    info!("start amqp to printer router");
    let start_options = StartOptions {
        scheduler: THREADING_SCHEDULER,
        router: ROUTER_BROADCAST,
        config_loader: CONFIG_LOADER_FILE,
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
