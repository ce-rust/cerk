use crate::runtime::{InternalServerFn, InternalServerId, ScheduleFn};

pub struct StartOptions {
    pub scheduler_start: ScheduleFn,
    pub router_start: InternalServerFn,
    pub config_loader_start: InternalServerFn,
    pub ports: Box<[(InternalServerId, InternalServerFn)]>,
}
