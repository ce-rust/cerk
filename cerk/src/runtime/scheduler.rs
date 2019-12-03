use crate::kernel::{KernelFn, StartOptions};

/// Function signature for the Scheduler.
pub type ScheduleFn = fn(StartOptions, KernelFn);
