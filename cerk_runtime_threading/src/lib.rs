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

# This Component: Threading Runtime

A Scheduler implementation for CERK based on the `std::thread` model.

`std::sync::mpsc` is used for the channels.

## Examples

* [Generator to MQTT](https://github.com/ce-rust/cerk/tree/master/examples/src/mqtt/)
* [Sequence to AMQP to Printer](https://github.com/ce-rust/cerk/tree/master/examples/src/sequence_to_amqp_to_printer/)
* [AMQP to Printer](https://github.com/ce-rust/cerk/tree/master/examples/src/amqp_to_printer/)

*/

#![deny(missing_docs)]

#[macro_use]
extern crate log;

pub mod channel;
mod scheduler;

pub use self::scheduler::threading_scheduler_start;
