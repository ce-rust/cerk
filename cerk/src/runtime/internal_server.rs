use super::channel::{BoxedReceiver, BoxedSender};

pub type InternalServerId = &'static str;
pub type InternalServerFn =
    fn(id: InternalServerId, inbox: BoxedReceiver, sender_to_kernel: BoxedSender);
