# cerk 0.1.0

[![Build status](https://badge.buildkite.com/4494e29d5f2c47e3fe998af46dff78a447800a76a68024e392.svg?branch=master)](https://buildkite.com/ce-rust/cerk)

[CERK](https://github.com/ce-rust/cerk) is an open source [CloudEvents](https://github.com/cloudevents/spec) Router written in Rust with a MicroKernel architecture.

## Introduction

CERK lets you route your [CloudEvents](https://github.com/cloudevents/spec) between different sources and sinks.
It is built with modularity and portability in mind.

## Components

CERK comes with a couple of prefabricated components, but implementing custom components is easy.

### Runtimes

### Ports

| Name                                                     | type   | Serialization    | Connection     |
|----------------------------------------------------------|--------|------------------|----------------|
| [port_input_unix_socket_json](./cerk_port_unix_socket/)  | input  | JSON             | UNIX Socket    |
| [port_output_unix_socket_json](./cerk_port_unix_socket/) | output | JSON             | UNIX Socket    |
| [port_sequence_generator](./cerk_port_dummies/)          | input  | -                | \<time based\> |
| [port_printer](./cerk_port_dummies/)                     | output | TEXT             |                |

### Routers

| Name                                                     | Description                        |
|----------------------------------------------------------|------------------------------------|
| [cerk_router_broadcast](./cerk_router_broadcast/)        | The broadcast router forwards all incomming CloudEvents to the configured ports |

### ConfigLoaders

| Name                                                     | Description                        |
|----------------------------------------------------------|------------------------------------|
| [static config loader](./examples/src/hello_world/main.rs)       | Have to be implemented for each project individually |

## Examples

| Name                                                     | Description                        |
|----------------------------------------------------------|------------------------------------|
| [Hello World](./examples/src/hello_world/)         | Routing CloudEvents that are generated from an input port to a output port, the output port print the result to the console. |
| [UNIX Socket](./examples/src/unix_socket/)         | Routing CloudEvents that are generated from an input port to a output port, the output port forwards the events to a UNIX socket |

## Update Readme

The original readme text is an rust doc comment in the [lib.rs](./cloudevents/src/lib.rs) file

1. `cargo install cargo-readme`
2. `cargo readme  -r cerk > README.md`

## License

Apache-2.0
