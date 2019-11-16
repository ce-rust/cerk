mod broker_event;
use crate::runtime::channel::{BoxedReceiver, BoxedSender};
use crate::runtime::ScheduFn;

pub use crate::kernel::broker_event::BrokerEvent;

struct Kernel {}

impl Kernel {
    fn start(start_options: StartOptions, inbox: BoxedReceiver, outbox: BoxedSender) {}
}

pub type KernelFn = fn(StartOptions, BoxedReceiver, BoxedSender);

pub struct StartOptions {
    pub scheduler_start: ScheduFn,
}

pub fn start_kernel(start_options: StartOptions) {
    (start_options.scheduler_start)(start_options, Kernel::start);
}
