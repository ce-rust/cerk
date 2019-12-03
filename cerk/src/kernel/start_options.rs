use crate::runtime::{InternalServerFn, InternalServerId, ScheduleFn};

/// The start option for the Kernel.
/// This struct defines the components that will be started with the scheduler.
pub struct StartOptions {
    /// the function to start the scheduler
    pub scheduler_start: ScheduleFn,

    /// the function to start the router
    pub router_start: InternalServerFn,

    /// the function to start the config loader
    pub config_loader_start: InternalServerFn,

    /// An array of port ids and functions to start the ports.
    /// That could handle input, output or both.
    /// The type of port depends on the messages the components send and receive.
    pub ports: Box<[(InternalServerId, InternalServerFn)]>,
}
