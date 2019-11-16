use crate::kernel::{KernelFn, StartOptions};

pub type ScheduFn = fn(StartOptions, KernelFn);

pub trait Scheduler {}
