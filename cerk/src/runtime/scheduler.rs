use crate::kernel::{KernelFn, StartOptions};

/// Function signature for the scheduler.
pub type ScheduFn = fn(StartOptions, KernelFn);
