# cerk_port_mqtt_mosquitto


This is a package for [CERK](https://github.com/ce-rust/cerk).
CERK is an open source [CloudEvents](https://github.com/cloudevents/spec) Router written in Rust with a MicroKernel architecture.

## Introduction

CERK lets you route your [CloudEvents](https://github.com/cloudevents/spec) between different different ports.
Ports are transport layer bindings over which CloudEvents can be exchanged.
It is built with modularity and portability in mind.

## Components

CERK comes with a couple of prefabricated components, but implementing custom components is easy.

A good overview is provided on [GitHub](https://github.com/ce-rust/cerk/).

## This Component: MQTT Port using libmosquitto

This port publishes and/or subscribe CloudEvents to/from an MQTT topic.

The port is implemented with the [Mosquitto Client](https://docs.rs/mosquitto-client/0.1.5/mosquitto_client/)
and sends and receives messages according to the CloudEvents specification (see [MQTT Protocol Binding for CloudEvents v1.0](https://github.com/cloudevents/spec/blob/v1.0/mqtt-protocol-binding.md))


## Configurations

The configurations should be of type `cerk::kernel::Config::HashMap` and have at least the entires:

### Required Fields

### host

The value has to by of type `Config::String` and contain a host name with protocol and port.

E.g. `Config::String(String::from("tcp://mqtt-broker:1883"))`

### Optional Fields

The following configurations are optional.

#### send_topic

The value has to by of type `Config::String` and contain the MQTT topic name where the message will be sent to.

E.g. `Config::String(String::from("inbox"))`

#### send_qos

The value has to by of type `Config::U8` and contain the [quality of service](http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718099) for message delivery.

Currently, the following values are supported:

* 0: At most once delivery (default)
* 1: At least once delivery

E.g. `Config::U8(1)`

### subscribe_topic

The value has to by of type `Config::String` and contain the MQTT topic which the router should subscribe to.

E.g. `Config::String(String::from("outbox"))`

### subscribe_qos

The value has to by of type `Config::U8` and contain the [quality of service](http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718099) for message delivery.

Currently, the following values are supported:

* 0: At most once delivery (default)
* 1: At least once delivery

E.g. `Config::U8(1)`

### Configuration Examples

#### Minimal Configuration to send events

This configuration will connect to the broker but will not send or receive any events.

```rust
use std::collections::HashMap;
use cerk::kernel::Config;

let map: HashMap<String, Config> = [
    ("host".to_string(), Config::String("tcp://mqtt-broker:1883".to_string())),
]
.iter()
.cloned()
.collect();

let config = Config::HashMap(map);
```

#### Full Configuration for sending events

```rust
use std::collections::HashMap;
use cerk::kernel::Config;

let map: HashMap<String, Config> = [
    ("host".to_string(), Config::String("tcp://mqtt-broker:1883".to_string())),
    ("send_topic".to_string(), Config::String("inbox".to_string())),
    ("send_qos".to_string(), Config::U8(1)),
]
.iter()
.cloned()
.collect();

let config = Config::HashMap(map);
```

#### Full Configuration for receiving events

```rust
use std::collections::HashMap;
use cerk::kernel::Config;

let map: HashMap<String, Config> = [
    ("host".to_string(), Config::String("tcp://mqtt-broker:1883".to_string())),
    ("subscribe_topic".to_string(), Config::String("outbox".to_string())),
    ("subscribe_qos".to_string(), Config::U8(1)),
]
.iter()
.cloned()
.collect();

let config = Config::HashMap(map);
```

#### Full Configuration for receiving and sending events

```rust
use std::collections::HashMap;
use cerk::kernel::Config;

let map: HashMap<String, Config> = [
    ("host".to_string(), Config::String("tcp://mqtt-broker:1883".to_string())),
    ("send_topic".to_string(), Config::String("inbox".to_string())),
    ("send_qos".to_string(), Config::U8(1)),
    ("subscribe_topic".to_string(), Config::String("outbox".to_string())),
    ("subscribe_qos".to_string(), Config::U8(1)),
]
.iter()
.cloned()
.collect();

let config = Config::HashMap(map);
```
