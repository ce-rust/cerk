mod receiver;
mod sender;

pub use crate::runtime::channel::receiver::Receiver;
pub use crate::runtime::channel::sender::Sender;

pub type BoxedReceiver = Box<dyn Receiver>;
pub type BoxedSender= Box<dyn Sender>;