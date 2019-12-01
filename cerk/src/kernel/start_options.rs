use crate::runtime::{InternalServerFn, InternalServerId, ScheduFn};

/// The start option for the Kernel. 
/// They define the components that will be started with the scheduler.
/// 
/// # Arguments
/// 
/// * `scheduler_start` - the function to start the scheduler
/// * `router_start` - the function to start the router
/// * `config_loader_start` - the function to start the config loader
/// * `ports` - An array of port ids and functions to start the ports. 
///    That could handle input, output or both. 
///    The type of port depends on the messages the components send and receive.
/// 
pub struct StartOptions {
    pub scheduler_start: ScheduFn,
    pub router_start: InternalServerFn,
    pub config_loader_start: InternalServerFn,
    pub ports: Box<[(InternalServerId, InternalServerFn)]>,
}
