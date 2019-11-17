pub use super::kernel_start::kernel_start;
use super::start_options::StartOptions;
use crate::runtime::channel::{BoxedReceiver, BoxedSender};

pub type KernelFn = fn(StartOptions, BoxedReceiver, BoxedSender);

pub fn bootstrap(start_options: StartOptions) {
    (start_options.scheduler_start)(start_options, kernel_start);
}
