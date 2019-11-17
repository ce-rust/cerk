#[macro_use]
extern crate log;
use cerk::runtime::{BoxedReceiver, BoxedSender};

pub fn config_loader_start(inbox: BoxedReceiver, sender_to_kernel: BoxedSender) {
    info!("start static config loader");
}
