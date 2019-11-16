pub mod channel;
mod internal_server;
mod scheduler;

pub use crate::runtime::channel::{BoxedReceiver, BoxedSender};
pub use crate::runtime::internal_server::InternalServer;
pub use crate::runtime::scheduler::ScheduFn;
pub type InternalServerFn = fn(inbox: BoxedReceiver, sender_to_kernel: BoxedSender);

pub type InternalServerId = &'static str;
