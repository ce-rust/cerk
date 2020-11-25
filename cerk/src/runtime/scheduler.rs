use crate::kernel::{KernelFn, StartOptions};

/// Function signature for the Scheduler.
pub type ScheduleFn = fn(StartOptions, KernelFn);
/// Function signature for the Scheduler as reference
pub type ScheduleFnRefStatic = &'static fn(StartOptions, KernelFn);
