/*!

This is a package for [CERK](https://github.com/ce-rust/cerk).
CERK is an open source [CloudEvents](https://github.com/cloudevents/spec) Router written in Rust with a MicroKernel architecture.

# Introduction

CERK lets you route your [CloudEvents](https://github.com/cloudevents/spec) between different different ports.
Ports are transport layer bindings over which CloudEvents can be exchanged.
It is built with modularity and portability in mind.

# Components

CERK comes with a couple of prefabricated components, but implementing custom components is easy.

A good overview is provided on [GitHub](https://github.com/ce-rust/cerk/).

# This Crate: UNIX Ports

These ports read or write CloudEvents from/to a UNIX Socket.

The ports are:

* port_input_unix_socket_json
* port_output_unix_socket_json

*/

#![deny(missing_docs)]

#[macro_use]
extern crate log;

mod port_input_unix_socket_json;
mod port_output_unix_socket_json;

pub use self::port_input_unix_socket_json::port_input_unix_socket_json_start;
pub use self::port_output_unix_socket_json::port_output_unix_socket_json_start;
