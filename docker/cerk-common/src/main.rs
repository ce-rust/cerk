#[macro_use]
extern crate cerk_loader_file;

use cerk_config_loader_file::CONFIG_LOADER_FILE;
use cerk_loader_file::{start, ComponentStartLinks};
use cerk_port_amqp::PORT_AMQP;
use cerk_port_dummies::{PORT_PRINTER, PORT_SEQUENCE_GENERATOR, PORT_SEQUENCE_VALIDATOR};
use cerk_port_health_check_http::PORT_HEALTH_CHECK_HTTP;
use cerk_port_mqtt::PORT_MQTT;
use cerk_port_unix_socket::{PORT_INPUT_UNIX_SOCKET, PORT_OUTPUT_UNIX_SOCKET};
use cerk_router_broadcast::ROUTER_BROADCAST;
use cerk_router_rule_based::ROUTER_RULE_BASED;
use cerk_runtime_threading::THREADING_SCHEDULER;

fn main() {
    env_logger::Builder::from_default_env().init();

    start(ComponentStartLinks {
        schedulers: fn_to_links![THREADING_SCHEDULER],
        routers: fn_to_links![ROUTER_BROADCAST, ROUTER_RULE_BASED],
        config_loaders: fn_to_links![CONFIG_LOADER_FILE],
        ports: fn_to_links![
            PORT_AMQP,
            PORT_MQTT,
            PORT_INPUT_UNIX_SOCKET,
            PORT_OUTPUT_UNIX_SOCKET,
            PORT_SEQUENCE_GENERATOR,
            PORT_SEQUENCE_VALIDATOR,
            PORT_PRINTER,
            PORT_HEALTH_CHECK_HTTP
        ],
    });
}
