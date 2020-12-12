# cerk_router_broadcast

[![Build status](https://badge.buildkite.com/4494e29d5f2c47e3fe998af46dff78a447800a76a68024e392.svg?branch=master)](https://buildkite.com/ce-rust/cerk)


This is a package for [CERK](https://github.com/ce-rust/cerk).
CERK is an open source [CloudEvents](https://github.com/cloudevents/spec) Router written in Rust with a MicroKernel architecture.

## Introduction

CERK lets you route your [CloudEvents](https://github.com/cloudevents/spec) between different different ports.
Ports are transport layer bindings over which CloudEvents can be exchanged.
It is built with modularity and portability in mind.

## Components

CERK comes with a couple of prefabricated components, but implementing custom components is easy.

A good overview is provided on [GitHub](https://github.com/ce-rust/cerk/).

## This Component: Broadcast Router

This router broadcasts all received CloudEvents to the configured ports.

## Configurations

The Socket expects a `Config::Vec([Config::String])` as configuration.
The strings should be Port ids, to which all received CloudEvents should be forwarded to.

e.g.
```rust
use cerk::kernel::Config;
let config = Config::Vec(vec![Config::String(String::from("output-port"))]);
```

## Examples

* [Hello World Example](https://github.com/ce-rust/cerk/tree/master/examples/src/hello_world)
* [UNIX Socket Example](https://github.com/ce-rust/cerk/tree/master/examples/src/unix_socket)
* [AMQP to Printer](https://github.com/ce-rust/cerk/tree/master/examples/src/amqp_to_printer/)
* [Sequence to AMQP to Printer](https://github.com/ce-rust/cerk/tree/master/examples/src/sequence_to_amqp_to_printer/)


## Update Readme

The original readme text is a Rust doc comment in the [lib.rs](./src/lib.rs) file

1. `cargo install cargo-readme`
2. `cargo readme  > README.md`

## License

Apache-2.0
