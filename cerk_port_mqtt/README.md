# cerk_port_mqtt

[![Build status](https://badge.buildkite.com/4494e29d5f2c47e3fe998af46dff78a447800a76a68024e392.svg?branch=master)](https://buildkite.com/ce-rust/cerk)


> :warning:  **this port currently supports the "Best Effort" delivery guarantee for incomming events**:
>
> The reason for this limitation is that the current version of the paho.mqtt.rust library acknowledges a received PUBLISH message automatically before the content of the message is handed over to the application.
>
> If a "At Least Once" delivery guarantee for incommming messages is required, the `cerk_port_mqtt_mosquitto` must be used.

This is a package for [CERK](https://github.com/ce-rust/cerk).
CERK is an open source [CloudEvents](https://github.com/cloudevents/spec) Router written in Rust with a MicroKernel architecture.

## Introduction

CERK lets you route your [CloudEvents](https://github.com/cloudevents/spec) between different different ports.
Ports are transport layer bindings over which CloudEvents can be exchanged.
It is built with modularity and portability in mind.

## Components

CERK comes with a couple of prefabricated components, but implementing custom components is easy.

A good overview is provided on [GitHub](https://github.com/ce-rust/cerk/).

## This Component: MQTT Port

This port publishes and/or subscribe CloudEvents to/from an MQTT v3.1 topic.

The port is implemented with a [Eclipse Paho MQTT Rust Client](https://github.com/eclipse/paho.mqtt.rust)
and sends and receives messages according to the
[MQTT Protocol Binding for CloudEvents v1.0](https://github.com/cloudevents/spec/blob/v1.0/mqtt-protocol-binding.md)
specification

## Configurations

The configurations should be of type `cerk::kernel::Config::HashMap` and have at least the entires:

### Required Fields

#### host

The value has to by of type `Config::String` and contain a host name with protocol and port.

E.g. `Config::String(String::from("tcp://mqtt-broker:1883"))`

### Optional Fields

#### send_topic

The value has to by of type `Config::String` and contain the MQTT topic name where the message will be sent to.

E.g. `Config::String(String::from("test"))`

The following configurations are optional.

#### subscribe_topic

The value has to by of type `Config::String` and contain the MQTT topic name  which the router should subscribe to.


### Configuration Examples

#### Configuration for sending and receiving events

```rust
use std::collections::HashMap;
use cerk::kernel::Config;

let map: HashMap<String, Config> = [
    ("host".to_string(), Config::String("tcp://mqtt-broker:1883".to_string())),
    ("subscribe_topic".to_string(), Config::String("inbox".to_string())),
    ("send_topic".to_string(), Config::String("outbox".to_string())),
]
.iter()
.cloned()
.collect();

let config = Config::HashMap(map);
```

#### Configuration for sending events

```rust
use std::collections::HashMap;
use cerk::kernel::Config;

let map: HashMap<String, Config> = [
    ("host".to_string(), Config::String("tcp://mqtt-broker:1883".to_string())),
    ("send_topic".to_string(), Config::String("outbox".to_string())),
]
.iter()
.cloned()
.collect();

let config = Config::HashMap(map);
```

#### Configuration for receiving events

```rust
use std::collections::HashMap;
use cerk::kernel::Config;

let map: HashMap<String, Config> = [
    ("host".to_string(), Config::String("tcp://mqtt-broker:1883".to_string())),
    ("subscribe_topic".to_string(), Config::String("inbox".to_string())),
]
.iter()
.cloned()
.collect();

let config = Config::HashMap(map);
```

## Examples

* [Generator to MQTT](https://github.com/ce-rust/cerk/tree/master/examples/examples/src/mqtt/)


## Update Readme

The original readme text is a Rust doc comment in the [lib.rs](./src/lib.rs) file

1. `cargo install cargo-readme`
2. `cargo readme  > README.md`

## License

Apache-2.0
