//! A channel implementation for CERK based on `std::sync::mpsc`.

mod new_channel;
mod receiver;
mod sender;

pub use self::new_channel::{new_channel_kernel_to_component, new_channel_with_size};
pub use self::receiver::ThreadingReceiver;
pub use self::sender::ThreadingSender;
