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

# This Crate: Dummy Ports

This crate contains some dummy ports for testing and demonstrations.

The ports are:

* port_printer
* port_sequence_generator

*/

#![deny(missing_docs)]

#[macro_use]
extern crate log;

mod port_printer;
mod port_sequence_generator;

pub use self::port_printer::port_printer_start;
pub use self::port_sequence_generator::port_sequence_generator_start;
