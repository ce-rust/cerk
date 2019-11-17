use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;

pub fn config_loader_start(
    id: InternalServerId,
    _inbox: BoxedReceiver,
    _sender_to_kernel: BoxedSender,
) {
    info!("start static config loader with id {}", id);
}
