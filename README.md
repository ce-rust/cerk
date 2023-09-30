# CERK

[![Build status](https://badge.buildkite.com/4494e29d5f2c47e3fe998af46dff78a447800a76a68024e392.svg?branch=master)](https://buildkite.com/ce-rust/cerk)
[![Crates.io](https://img.shields.io/crates/v/cerk)](https://docs.rs/cerk/*/cerk/)
[![Docs status](https://docs.rs/cerk/badge.svg)](https://docs.rs/cerk/)

[CERK](https://github.com/ce-rust/cerk) is an open-source [CloudEvents](https://github.com/cloudevents/spec) Router written in Rust with a MicroKernel architecture.

## Introduction

CERK lets you route your [CloudEvents](https://github.com/cloudevents/spec) between different different ports.
Ports are transport layer bindings over which CloudEvents can be exchanged.
It is built with modularity and portability in mind.

The project was initially created during the student research project [CloudEvents Router](https://eprints.hsr.ch/832/) and extended in the bachelor thesis [Reliable Messaging using the CloudEvents Router](https://eprints.ost.ch/id/eprint/904/).

The conference paper [Reliable Event Routing in the Cloud and on the Edge: An Internet-of-Things Solution in the AgeTech Domain](https://doi.org/10.1007/978-3-030-86044-8_17) describes a specific problem and the solution that was addressed with CERK.


## Get Started

The easiest way to use the router is to use the [docker image](https://github.com/orgs/ce-rust/packages?repo_name=cerk).

If you like to build the router by yourself, start with an [example](#examples). E.g. the [hello world example](https://github.com/ce-rust/cerk/tree/master/examples/examples/src/hello_world).


## Components

CERK comes with a couple of prefabricated components, but implementing custom components is easy.

### MicroKernel

The MicroKernel is responsible for starting the other components with the help of the Scheduler and brokering messages between them.

The MicroKernel is implemented in the [`cerk`](./cerk/) crate.

### Runtimes

The Runtime provides a Scheduler and a Channel (Sender/Receiver) implementation.

The Scheduler is responsible for scheduling the internal servers with a platform specific scheduling strategy.

| Name                                                 | Scheduling Strategy | Channel Strategy    | Compatible with |
|------------------------------------------------------|---------------------|---------------------|-----------------|
| [cerk_runtime_threading](./cerk_runtime_threading/)  | `std::thread`       | `std::sync::mpsc`   | Linux / MacOS   |

### Ports

The Port is responsible for exchanging CloudEvents with the outside world.
A Port can be instantiated multiple times with different configurations.

| Name                                                     | type          | Serialization    | Connection     |
|----------------------------------------------------------|---------------|------------------|----------------|
| [port_input_unix_socket_json](./cerk_port_unix_socket/)  | input         | JSON             | UNIX Socket    |
| [port_output_unix_socket_json](./cerk_port_unix_socket/) | output        | JSON             | UNIX Socket    |
| [port_mqtt](./cerk_port_mqtt/)                           | input/output  | JSON             | MQTT           |
| [port_mqtt_mosquitto](./cerk_port_mqtt_mosquitto/)       | input/output  | JSON             | MQTT           |
| [port_amqp](./cerk_port_amqp)                            | input/output  | JSON             | AMQP           |
| [port_sequence_generator](./cerk_port_dummies/)          | input         | -                | \<time based\> |
| [port_printer](./cerk_port_dummies/)                     | output        | TEXT             |                |

### Routers

The Router is responsible for deciding to which port a received CloudEvent should be forwarded to.

| Name                                                     | Description                        |
|----------------------------------------------------------|------------------------------------|
| [cerk_router_broadcast](./cerk_router_broadcast/)        | The broadcast router forwards all incoming CloudEvents to the configured ports. |
| [cerk_router_rule_based](./cerk_router_rule_based/)      | The rule-based router routes events based on the given configuration. The configurations are structured in a tree format. One configuration tree per output port needs to be configured. The operations  `And`, `Or`, `Contains`, `StartsWith` and more are supported. |

### ConfigLoaders

The ConfigLoader is responsible for providing the newest port configurations and routing rules.

| Name                                                             | Description                                          |
|------------------------------------------------------------------|------------------------------------------------------|
| [config_loader_file](./cerk_config_loader_file/)                 | Loads the configurations from a json based file      |
| [config loader_static](./examples/examples/src/hello_world/main.rs)       | Have to be implemented for each project individually |

### Loaders

The Loader helps by starting the router with different components.

| Name                                                             | Description                                          |
|------------------------------------------------------------------|------------------------------------------------------|
| [cerk_loader_file](./cerk_loader_file/)                          | Starts the router by configuration provided by a json file |


### Health Check Ports

The Health Check Port provides an interface to check if the router is healthy.
Technically they use the same plugin interface as the ports; however, they do not send or receive CloudEvents.

| Name                                                             | Interface                                            |
|------------------------------------------------------------------|------------------------------------------------------|
| [cerk_port_health_check_http](./cerk_port_health_check_http/)    | HTTP                                                 |


## Examples

Check out the README in the folder of each example for more details and setup instructions.

| Name                                                             | Description                        |
|------------------------------------------------------------------|------------------------------------|
| [Hello World](./examples/examples/src/hello_world/)                       | Routing CloudEvents that are generated from an input port to an output port, the output port print the result to the console. |
| [Rule Based Routing Example](./examples/examples/src/rule_based_routing/) | CloudEvents that are generated from an input port are routed to an output port, but in this example only every tenth event gets routed to the output port because they are filtered by `id`. The `id` has to end with `0`, thus only 10,20,30,... are printed. |
| [UNIX Socket](./examples/examples/src/unix_socket/)                       | Routes CloudEvents from an input UNIX Socket port to an output UNIX Socket port |
| [MQTT](./examples/examples/src/mqtt/)                                     | Routes CloudEvents that are generated from an input port to an output port, the output port publishes the events on an MQTT topic. A second router subscribes to the same topic with an MQTT port and routes them to a port which prints the event to stdout. |
| [AMQP to Printer](./examples/examples/src/amqp_to_printer/)               | Routes CloudEvents from a RabbitMQ exchange to a queue to CERK and finally prints them to the console. |
| [Sequence to AMQP to Printer](./examples/examples/src/sequence_to_amqp_to_printer/)   | The setup contains two routers. One Router generates events and routs them to a RabbitMQ exchange. Another router consumes the CloudEvents from a bound queue and prints them to the console. |
| [UNIX Socket and MQTT for armv7](./examples/unix_socket_and_mqtt_on_armv7/) | Routes CloudEvents that are received on an input UNIX Socket port to an output UNIX Socket port and an MQTT output port. |

## Delivery Guarantees

The router supports two delivery guarantees: `BestEffort` and `AtLeastOnce`

The delivery guarantee is defined on the incoming port and is attached to each message that gets transferred through the router.

## Development Setup

Different Docker-based development environments can be found [here](https://github.com/ce-rust/cerk/tree/master/setup).

### Prerequisites without Docker

1. latest version of [rustup](https://www.rust-lang.org/tools/install)
2. Rust version 1.72.0: `rustup install 1.72`
3. Additional C libraries depending on the platform (examples can be found in the Docker containers)

Optional Tooling:
1. rustfmt: `rustup component add rustfmt`
2. cargo-readme: `cargo install cargo-readme`

### Run Tests

```bash
cargo test --all
```

### Format Code

```bash
cargo fmt --all
```

### Generate Documentation

```bash
cargo doc --no-deps --open
```


## Update a Crate Readme

1. `cargo install cargo-readme`
2. `cd <crate>`
3. `cargo readme > README.md`

## Release Management

Release management is organized by cargo-workspaces.

pre-requirement: `cargo install cargo-workspaces`

1. check out master and create a new branch `release`
2. `cargo workspaces publish --allow-branch="release" --no-git-push`
   sometimes the publish command hangs ... if so, you could repeat the cargo publish with the script `./publish-packages.sh`
3. update the dependencies of the docker container `(cd docker/cerk-common; cargo update)`
4. merge it back into the master with a pull request

## License

Apache-2.0
