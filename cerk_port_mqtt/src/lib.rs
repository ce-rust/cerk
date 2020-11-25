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

# This Component: MQTT Port

This port publishes and/or subscribe CloudEvents to/from an MQTT v3.1 topic.

The port is implemented with a [Eclipse Paho MQTT Rust Client](https://github.com/eclipse/paho.mqtt.rust)
and sends and receives messages according to the
[MQTT Protocol Binding for CloudEvents v1.0](https://github.com/cloudevents/spec/blob/v1.0/mqtt-protocol-binding.md)
specification

# Configurations

The configurations should be of type `cerk::kernel::Config::HashMap` and have at least the entires:

## Required Fields

## host

The value has to by of type `Config::String` and contain a host name with protocol and port.

E.g. `Config::String(String::from("tcp://mqtt-broker:1883"))`

## Optional Fields

### send_topic

The value has to by of type `Config::String` and contain the MQTT topic name where the message will be sent to.

E.g. `Config::String(String::from("test"))`

The following configurations are optional.

### persistence

The value has to by of type `Config::U8` and contain one of the following values.

The values are defined according to the Eclipse Paho MQTT Rust Client PersistenceType.

* 0: File (default) -  Data and messages are persisted to a local file (default)
* 1: None - No persistence is used.

E.g. `Config::U8(0)`

### send_qos

The [quality of service](http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718099) for message delivery.
The quality of service is only for the MQTT broker and does not change any behavior of the router or the port.
The router only supports best effort at the moment.

* 0: At most once delivery (default)
* 1: At least once delivery
* 2: Exactly once delivery

E.g. `Config::U8(0)`

## subscribe_topics

The value has to by of type `Config::Vec([Config::String])` and must have the same length as `subscribe_qos`.
The values in the vector contain the MQTT topic wich the router should subscribe to.

If multiple topics are subscribed in the same MQTT port,
there is no possibility at the moment to know let the router or the output port know from which topic the event was received.

## subscribe_qos

The value has to by of type `Config::Vec([Config::U8])` and must have the same length as `subscribe_topics`.

The [quality of service](http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718099) for the topic subscription.
The quality of service is only for the MQTT broker and does not change any behavior of the router or the port.
The router only supports best effort at the moment.

* 0: At most once delivery
* 1: At least once delivery
* 2: Exactly once delivery

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
    ("persistence".to_string(), Config::U8(0)),
    ("send_topic".to_string(), Config::String("test".to_string())),
    ("send_qos".to_string(), Config::U8(2)),
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
    ("persistence".to_string(), Config::U8(0)),
    (
      "subscribe_topics".to_string(),
      Config::Vec(vec![Config::String("test".to_string())]),
    ),
    (
      "subscribe_qos".to_string(),
      Config::Vec(vec![Config::U8(2)]),
    ),
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
    ("persistence".to_string(), Config::U8(0)),
    ("send_topic".to_string(), Config::String("test".to_string())),
    ("send_qos".to_string(), Config::U8(2)),
    (
      "subscribe_topics".to_string(),
      Config::Vec(vec![Config::String("test".to_string())]),
    ),
    (
      "subscribe_qos".to_string(),
      Config::Vec(vec![Config::U8(2)]),
    ),
]
.iter()
.cloned()
.collect();

let config = Config::HashMap(map);
```

# Examples

* [Generator to MQTT](https://github.com/ce-rust/cerk/tree/master/examples/src/mqtt/)

# Limitations

* **reliability** this port does not support any `DeliveryGuarantee` other then `Unspecified` and so does never send a `OutgoingCloudEventProcessed` or `IncomingCloudEventProcessed` messages

*/

#![deny(missing_docs)]

#[macro_use]
extern crate log;

mod port_mqtt;

pub use self::port_mqtt::{port_mqtt_start, PORT_MQTT};
