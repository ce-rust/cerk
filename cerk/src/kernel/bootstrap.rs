pub use super::kernel_start::kernel_start;
use super::start_options::StartOptions;
use crate::runtime::channel::{BoxedReceiver, BoxedSender};

/// Function signature for the Kernel.
///
/// # Arguments
///
/// * `start_options` - The start options defining the components and the behavior of the router.
/// * `inbox` - The inbox channel of the Kernel, every component should send there events for the Kernel to this channel.
/// * `sender_to_scheduler` - The outbox channel for messages for the Scheduler.
///
pub type KernelFn =
    fn(start_options: StartOptions, inbox: BoxedReceiver, sender_to_scheduler: BoxedSender);

/// The `bootstrap` function is the entrance point of the CERK router.
/// This function starts the Kernel with the help of the scheduler.
/// Later, the Kernel starts all components, and the router starts working.
///
/// # Arguments
///
/// * `start_options` - The start options defining the components and the behavior of the router.
///
pub fn bootstrap(start_options: StartOptions) {
    (start_options.scheduler)(start_options, kernel_start);
}
