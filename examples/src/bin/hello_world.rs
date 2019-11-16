use cerk::kernel::{start_kernel, StartOptions};
use cerk_config_loader_static::config_loader_start;
use cerk_router_broadcast::start_routing;
use cerk_runtime_threading::ThreadingScheduler;

fn main() {
    let start_options = StartOptions {
        scheduler_start: ThreadingScheduler::start,
        router_start: start_routing,
        config_loader_start: config_loader_start,
        ports: Box::new([]),
    };
    start_kernel(start_options);
}
