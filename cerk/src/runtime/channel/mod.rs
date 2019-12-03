//! Wrappers for platform-specific channel implementation used in CERK.

mod receiver;
mod sender;

pub use self::receiver::{BoxedReceiver, Receiver};
pub use self::sender::{BoxedSender, Sender};
