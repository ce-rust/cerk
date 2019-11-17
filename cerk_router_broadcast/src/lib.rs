#[macro_use]
extern crate log;
use cerk::runtime::{BoxedReceiver, BoxedSender};

pub fn start_routing(inbox: BoxedReceiver, sender_to_kernel: BoxedSender) {
    info!("start broadcast router");
}
