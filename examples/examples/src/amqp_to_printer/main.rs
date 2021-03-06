#[macro_use]
extern crate cerk_loader_file;

use env_logger::Env;

use cerk_config_loader_file::CONFIG_LOADER_FILE;
use cerk_loader_file::{start, ComponentStartLinks};
use cerk_port_amqp::PORT_AMQP;
use cerk_port_dummies::PORT_PRINTER;
use cerk_port_health_check_http::PORT_HEALTH_CHECK_HTTP;
use cerk_router_broadcast::ROUTER_BROADCAST;
use cerk_runtime_threading::THREADING_SCHEDULER;

fn main() {
    env_logger::from_env(Env::default().default_filter_or("debug")).init();

    start(ComponentStartLinks {
        schedulers: fn_to_links![THREADING_SCHEDULER],
        routers: fn_to_links![ROUTER_BROADCAST],
        config_loaders: fn_to_links![CONFIG_LOADER_FILE],
        ports: fn_to_links![PORT_PRINTER, PORT_AMQP, PORT_HEALTH_CHECK_HTTP],
    });
}
