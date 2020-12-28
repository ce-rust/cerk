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

# This Component: AMQP Port

This port publishes and/or subscribe CloudEvents to/from an AMQP broker with protocol version v0.9.1.

The port is implemented with [lapin](https://github.com/CleverCloud/lapin).

## Content Modes

The port supports the structured content mode with the JSON event format.
However, it does not support the binary content mode.

<https://github.com/cloudevents/spec/blob/master/amqp-protocol-binding.md#2-use-of-cloudevents-attributes>

## Examples

 * [Sequence to AMQP to Printer](https://github.com/ce-rust/cerk/tree/master/examples/examples/src/sequence_to_amqp_to_printer/)
 * [AMQP to Printer](https://github.com/ce-rust/cerk/tree/master/examples/examples/src/amqp_to_printer/)

*/

#![deny(missing_docs)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

pub mod lapin_helper;
mod port_amqp;

pub use self::port_amqp::{port_amqp_start, PORT_AMQP};
