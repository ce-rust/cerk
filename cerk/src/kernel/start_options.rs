use crate::kernel::ScheduleInternalServerStatic;
use crate::runtime::{InternalServerFnRefStatic, ScheduleFnRefStatic};

/// The start option for the Kernel.
/// This struct defines the components that will be started with the scheduler.
pub struct StartOptions {
    /// the function to start the scheduler
    pub scheduler: ScheduleFnRefStatic,

    /// the function to start the router
    pub router: InternalServerFnRefStatic,

    /// the function to start the config loader
    pub config_loader: InternalServerFnRefStatic,

    /// A vector of port ids and functions to start the ports.
    /// That could handle input, output or both.
    /// The type of port depends on the messages the components send and receive.
    pub ports: Vec<ScheduleInternalServerStatic>,
}
