use cerk::runtime::channel::{BoxedReceiver, BoxedSender};

pub fn port_printer_start(inbox: BoxedReceiver, sender_to_kernel: BoxedSender) {}
pub fn port_sequence_generator_start(inbox: BoxedReceiver, sender_to_kernel: BoxedSender) {}
