//! Runtime definitions for CERK.
//! The implementations for the declarations could be platform-specific.

pub mod channel;
mod internal_server;
mod scheduler;

pub use self::internal_server::{
    InternalServerFn, InternalServerFnRef, InternalServerFnRefStatic, InternalServerId,
};
pub use self::scheduler::{ScheduleFn, ScheduleFnRefStatic};
