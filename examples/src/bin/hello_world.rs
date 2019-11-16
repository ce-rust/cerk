use cerk::kernel::{start_kernel, StartOptions};
use cerk_runtime_threading::ThreadingScheduler;
fn main() {
    let start_options = StartOptions {
        scheduler_start: ThreadingScheduler::start,
    };
    start_kernel(start_options);
}
