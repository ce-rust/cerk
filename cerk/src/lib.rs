/*!
[CERK](https://github.com/ce-rust/cerk) is an open-source [CloudEvents](https://github.com/cloudevents/spec) Router written in Rust with a MicroKernel architecture.

# Introduction

CERK lets you route your [CloudEvents](https://github.com/cloudevents/spec) between different different ports.
Ports are transport layer bindings over which CloudEvents can be exchanged.
It is built with modularity and portability in mind.

# Components

CERK comes with a couple of prefabricated components, but implementing custom components is easy.

## MicroKernel

The MicroKernel is responsible for starting the other components with the help of the Scheduler and brokering messages between them.

The MicroKernel is implemented in the [`cerk`](./cerk/) crate.

## Runtimes

The Runtime provides a Scheduler and a Channel (Sender/Receiver) implementation.

The Scheduler is responsible for scheduling the internal servers with a platform specific scheduling strategy.

| Name                                                 | Scheduling Strategy | Channel Strategy    | Compatible with |
|------------------------------------------------------|---------------------|---------------------|-----------------|
| [cerk_runtime_threading](./cerk_runtime_threading/)  | `std::thread`       | `std::sync::mpsc`   | Linux / MacOS   |

## Ports

The Port is responsible for exchanging CloudEvents with the outside world.
A Port can be instantiated multiple times with different configurations.

| Name                                                     | type          | Serialization    | Connection     |
|----------------------------------------------------------|---------------|------------------|----------------|
| [port_input_unix_socket_json](./cerk_port_unix_socket/)  | input         | JSON             | UNIX Socket    |
| [port_output_unix_socket_json](./cerk_port_unix_socket/) | output        | JSON             | UNIX Socket    |
| [port_mqtt](./cerk_port_mqtt/)                           | input/output  | JSON             | MQTT           |
| [port_sequence_generator](./cerk_port_dummies/)          | input         | -                | \<time based\> |
| [port_printer](./cerk_port_dummies/)                     | output        | TEXT             |                |

## Routers

The Router is responsible for deciding to which port a received CloudEvent should be forwarded to.

| Name                                                     | Description                        |
|----------------------------------------------------------|------------------------------------|
| [cerk_router_broadcast](./cerk_router_broadcast/)        | The broadcast router forwards all incomming CloudEvents to the configured ports. |
| [cerk_router_rule_based](./cerk_router_rule_based/)      | The rule-based router routes events based on the given configuration. The configurations are structured in a tree format. One configuration tree per output port needs to be configured. The operations  `And`, `Or`, `Contains`, `StartsWith` and more are supported. |

## ConfigLoaders

The ConfigLoader is responsible for providing the newest port configurations and routing rules.

| Name                                                             | Description                                          |
|------------------------------------------------------------------|------------------------------------------------------|
| [static config loader](./examples/src/hello_world/main.rs)       | Have to be implemented for each project individually |

# Examples

Check out the README in the folder of each example for more details and setup instructions.

| Name                                                             | Description                        |
|------------------------------------------------------------------|------------------------------------|
| [Hello World](./examples/src/hello_world/)                       | Routing CloudEvents that are generated from an input port to an output port, the output port print the result to the console. |
| [Rule Based Routing Example](./examples/src/rule_based_routing/) | CloudEvents that are generated from an input port are routed to an output port, but in this example only every tenth event gets routed to the output port because they are filterd by `id`. The `id` has to end with `0`, thus only 10,20,30,... are printed. |
| [UNIX Socket](./examples/src/unix_socket/)                       | Routes CloudEvents from an input UNIX Socket port to an output UNIX Socket port |
| [MQTT](./examples/src/sequence_to_mqtt/)                         | Routes CloudEvents that are generated from an input port to an output port, the output port publishes the events on an MQTT topic. A second router subscribes to the same topic with an MQTT port and routes them to a port which prints the event to stdout. |
| [UNIX Socket and MQTT for armv7](./examples/src/unix_socket_and_mqtt_on_armv7/) | Routes CloudEvents that are received on an input UNIX Socket port to an output UNIX Socket port and an MQTT output port. |

# Development Setup

Different Docker-based development environments can be found [here](https://github.com/ce-rust/cerk/tree/master/setup).

## Prerequisites without Docker

1. latest version of [rustup](https://www.rust-lang.org/tools/install)
2. Rust version 1.38.0: `rustup install 1.38.0`

Optional Tooling:
1. rustfmt: `rustup component add rustfmt`
2. cargo-readme: `cargo install cargo-readme`

## Run Tests

```rust
cargo test --all
```

## Format Code

```rust
cargo fmt --all
```

## Generate Documentation

```rust
cargo doc --no-deps --open
```

*/

#![deny(missing_docs)]

#[macro_use]
extern crate log;

pub mod kernel;
pub mod runtime;
