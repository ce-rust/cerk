use crate::kernel::{KernelFn, StartOptions};

/// Function signature for the Scheduler.
pub type ScheduFn = fn(StartOptions, KernelFn);
