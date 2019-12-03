use crate::kernel::{KernelFn, StartOptions};

pub type ScheduleFn = fn(StartOptions, KernelFn);
