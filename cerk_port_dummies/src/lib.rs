/*!
This is a package for [CERK](https://github.com/ce-rust/cerk).
CERK is an open source [CloudEvents](https://github.com/cloudevents/spec) Router written in Rust with a MicroKernel architecture.

This package contains some dummy ports for testing and demonstrations.

* [port_printer](./port_printer.rs)
* [port_sequence_generator](./port_sequence_generator.rs)

*/

#![deny(missing_docs)]

#[macro_use]
extern crate log;

mod port_printer;
mod port_sequence_generator;

pub use self::port_printer::port_printer_start;
pub use self::port_sequence_generator::port_sequence_generator_start;
