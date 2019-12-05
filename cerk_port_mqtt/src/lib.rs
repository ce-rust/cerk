/*!
This is a package for [CERK](https://github.com/ce-rust/cerk).
CERK is an open source [CloudEvents](https://github.com/cloudevents/spec) Router written in Rust with a MicroKernel architecture.
*/

#![deny(missing_docs)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate cloudevents;

mod port_output_mqtt;

pub use self::port_output_mqtt::port_output_mqtt_start;
