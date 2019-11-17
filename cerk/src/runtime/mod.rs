pub mod channel;
mod internal_server;
mod scheduler;

pub use crate::runtime::channel::{BoxedReceiver, BoxedSender};
pub use crate::runtime::internal_server::InternalServer;
pub use crate::runtime::scheduler::ScheduFn;

pub type InternalServerId = &'static str;
pub type InternalServerFn =
    fn(id: InternalServerId, inbox: BoxedReceiver, sender_to_kernel: BoxedSender);
