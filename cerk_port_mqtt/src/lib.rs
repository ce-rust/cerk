/*!

> :warning:  **this port currently supports the "Best Effort" delivery guarantee for incomming events**:
>
> The reason for this limitation is that the current version of the paho.mqtt.rust library acknowledges a received PUBLISH message automatically before the content of the message is handed over to the application.
>
> If a "At Least Once" delivery guarantee for incommming messages is required, the `cerk_port_mqtt_mosquitto` must be used.

This is a package for [CERK](https://github.com/ce-rust/cerk).
CERK is an open source [CloudEvents](https://github.com/cloudevents/spec) Router written in Rust with a MicroKernel architecture.

# Introduction

CERK lets you route your [CloudEvents](https://github.com/cloudevents/spec) between different different ports.
Ports are transport layer bindings over which CloudEvents can be exchanged.
It is built with modularity and portability in mind.

# Components

CERK comes with a couple of prefabricated components, but implementing custom components is easy.

A good overview is provided on [GitHub](https://github.com/ce-rust/cerk/).

# This Component: MQTT Port

This port publishes and/or subscribe CloudEvents to/from an MQTT v3.1 topic.

The port is implemented with a [Eclipse Paho MQTT Rust Client](https://github.com/eclipse/paho.mqtt.rust)
and sends and receives messages according to the
[MQTT Protocol Binding for CloudEvents v1.0](https://github.com/cloudevents/spec/blob/v1.0/mqtt-protocol-binding.md)
specification

# Configurations

The configurations should be of type `cerk::kernel::Config::HashMap` and have at least the entires:

## Required Fields

### host

The value has to by of type `Config::String` and contain a host name with protocol and port.

E.g. `Config::String(String::from("tcp://mqtt-broker:1883"))`

## Optional Fields

### send_topic

The value has to by of type `Config::String` and contain the MQTT topic name where the message will be sent to.

E.g. `Config::String(String::from("test"))`

The following configurations are optional.

### subscribe_topic

The value has to by of type `Config::String` and contain the MQTT topic name  which the router should subscribe to.


## Configuration Examples

### Minimal Configuration to send events

This configuration will connect to the broker but will not send or receive any events.

```
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

### Full Configuration for sending events

```
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

### Full Configuration for recieve events

```
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

### Full Configuration for receiving events

```
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

# Examples

* [Generator to MQTT](https://github.com/ce-rust/cerk/tree/master/examples/src/mqtt/)

*/

#![deny(missing_docs)]

#[macro_use]
extern crate log;

mod port_mqtt;

pub use self::port_mqtt::{port_mqtt_start, PORT_MQTT};
