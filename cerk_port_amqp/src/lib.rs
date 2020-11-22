/*!
This is a package for [CERK](https://github.com/ce-rust/cerk).
CERK is an open source [CloudEvents](https://github.com/cloudevents/spec) Router written in Rust with a MicroKernel architecture.
*/

#![deny(missing_docs)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

pub mod lapin_helper;
mod port_amqp;

pub use self::port_amqp::port_amqp_start;
