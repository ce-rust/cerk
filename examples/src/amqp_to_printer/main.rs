#[macro_use]
extern crate log;

use env_logger::Env;

use cerk::kernel::{bootstrap, StartOptions};
use cerk_config_loader_file::config_loader_file_start;
use cerk_port_amqp::port_amqp_start;
use cerk_port_dummies::port_printer_start;
use cerk_router_broadcast::router_start;
use cerk_runtime_threading::threading_scheduler_start;

const AMQP_INPUT: &'static str = "ampq-input";
const DUMMY_LOGGER_OUTPUT: &'static str = "dummy-logger-output";

fn main() {
    env_logger::from_env(Env::default().default_filter_or("debug")).init();
    info!("start amqp to printer router");
    let start_options = StartOptions {
        scheduler_start: threading_scheduler_start,
        router_start: router_start,
        config_loader_start: config_loader_file_start,
        ports: Box::new([
            (String::from(AMQP_INPUT), port_amqp_start),
            (String::from(DUMMY_LOGGER_OUTPUT), port_printer_start),
        ]),
    };
    bootstrap(start_options);
}
