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

# This Component: Broadcast Router

This router broadcasts all received CloudEvents to the configured ports.

# Configurations

The Socket expects a `Config::Vec([Config::String])` as configuration.
The strings should be Port ids, to which all received CloudEvents should be forwarded to.

e.g.
```
use cerk::kernel::Config;
let config = Config::Vec(vec![Config::String(String::from("output-port"))]);
```

# Examples

* [Hello World Example](https://github.com/ce-rust/cerk/tree/master/examples/src/hello_world)
* [UNIX Socket Example](https://github.com/ce-rust/cerk/tree/master/examples/src/unix_socket)
* [AMQP to Printer](https://github.com/ce-rust/cerk/tree/master/examples/src/amqp_to_printer/)
* [Sequence to AMQP to Printer](https://github.com/ce-rust/cerk/tree/master/examples/src/sequence_to_amqp_to_printer/)

*/

#![deny(missing_docs)]

#[macro_use]
extern crate log;

mod router;

pub use self::router::{router_start, ROUTER_BROADCAST};
