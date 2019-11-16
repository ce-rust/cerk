use crate::kernel::StartOptions;

pub type ScheduFn = fn(start_options: StartOptions);

pub trait Scheduler {}
