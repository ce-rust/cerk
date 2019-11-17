#[macro_use]
extern crate log;
use cerk::runtime::InternalServerId;
use cerk::runtime::{BoxedReceiver, BoxedSender};

pub fn config_loader_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    sender_to_kernel: BoxedSender,
) {
    info!("start static config loader with id {}", id);
}
