use crate::runtime::{InternalServerFn, InternalServerId, ScheduFn};

pub struct StartOptions {
    pub scheduler_start: ScheduFn,
    pub router_start: InternalServerFn,
    pub config_loader_start: InternalServerFn,
    pub ports: Box<[(InternalServerId, InternalServerFn)]>,
}
