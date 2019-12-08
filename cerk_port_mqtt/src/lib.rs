/*!
This is a package for [CERK](https://github.com/ce-rust/cerk).
CERK is an open source [CloudEvents](https://github.com/cloudevents/spec) Router written in Rust with a MicroKernel architecture.
*/

#![deny(missing_docs)]

#[macro_use]
extern crate log;

mod port_mqtt;

pub use self::port_mqtt::port_mqtt_start;
