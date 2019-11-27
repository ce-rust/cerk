# CERK - CloudEvents Router Kernel

[![Build status](https://badge.buildkite.com/4494e29d5f2c47e3fe998af46dff78a447800a76a68024e392.svg?branch=master)](https://buildkite.com/ce-rust/cerk)

CERK is an open source [CloudEvents](https://github.com/cloudevents/spec) Router written in Rust with a MicroKernel architecture. 

## Introduction

CERK lets you route your [CloudEvents](https://github.com/cloudevents/spec) between different sources and sinks.
It is built with modularity and portability in mind.

## Components

CERK comes with a couple of prefabricated components, but implementing custom components is easy.

### Runtimes

### Ports

| Name                                                     | Serialization    | Connection     |
|----------------------------------------------------------|------------------|----------------|
| [port_output_unix_socket_json](./cerk_port_unix_socket/) | JSON             | UNIX Socket    |
| [port_sequence_generator](./cerk_port_dummies/)          | -                | \<time based\> |
| [port_printer](./cerk_port_dummies/)                     | TEXT             |                |

### Routers

| Name                                                     | Description                        |
|----------------------------------------------------------|------------------------------------|
| [cerk_router_broadcast](./cerk_router_broadcast/)       | The broadcast router forwards all incomming CloudEvents to the configured ports |

### ConfigLoaders

| Name                                                     | Description                        |
|----------------------------------------------------------|------------------------------------|
| [static config loader](./examples/src/bin/hello_world.rs)       | Have to be implemented for each project individually |


## Examples

| Name                                                     | Description                        |
|----------------------------------------------------------|------------------------------------|
| [Hello World](./examples/src/bin/hello_world.rs)         | Routing CloudEvents that are generated from an input port to a output port, the output port print the result to the console. |
| [UNIX Socket](./examples/src/bin/unix_socket.rs)         | Routing CloudEvents that are generated from an input port to a output port, the output port forwards the events to a UNIX socket |


## CloudEvents

More details about CloudEvents can be found [here](./cloudevents/README.md).
