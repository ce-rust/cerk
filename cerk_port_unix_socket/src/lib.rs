/*!
This is a package for [CERK](https://github.com/ce-rust/cerk).
CERK is an open source [CloudEvents](https://github.com/cloudevents/spec) Router written in Rust with a MicroKernel architecture.
*/

#![deny(missing_docs)]

#[macro_use]
extern crate log;

mod port_input_unix_socket_json;
mod port_output_unix_socket_json;

pub use self::port_input_unix_socket_json::port_input_unix_socket_json_start;
pub use self::port_output_unix_socket_json::port_output_unix_socket_json_start;
