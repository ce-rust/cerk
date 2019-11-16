mod broker_event;
use crate::runtime::ScheduFn;

pub use crate::kernel::broker_event::BrokerEvent;

pub struct StartOptions {
    pub scheduler_start: ScheduFn,
}

pub fn start_kernel(start_options: StartOptions) {
    (start_options.scheduler_start)(start_options);
}
