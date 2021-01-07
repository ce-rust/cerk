/*!

> :warning:  **this port requires a special build of libmosquitto to be present locally**: [`feat/reliable` branch of https://github.com/ce-rust/mosquitto](https://github.com/ce-rust/mosquitto/tree/feat/reliable)
>
> The reason for this limitation is that the current version of libmosquitto acknowledges a received PUBLISH message automatically before the content of the message is handed over to the application.
> This patch needs to stay until this issue is fixed upstream (see https://github.com/eclipse/mosquitto/pull/1932).
>
> This patch could be used by adding the following two parameters to the cargo build command:
>  * `MOSQUITTO_GIT_URL=https://github.com/ce-rust/mosquitto`
>  * `MOSQUITTO_GIT_HASH=9f834dff9095e6731937d5eac767dbaca46491ac`
> All the docker compose setups use this binary.

This is a package for [CERK](https://github.com/ce-rust/cerk).
CERK is an open source [CloudEvents](https://github.com/cloudevents/spec) Router written in Rust with a MicroKernel architecture.

# Introduction

CERK lets you route your [CloudEvents](https://github.com/cloudevents/spec) between different different ports.
Ports are transport layer bindings over which CloudEvents can be exchanged.
It is built with modularity and portability in mind.

# Components

CERK comes with a couple of prefabricated components, but implementing custom components is easy.

A good overview is provided on [GitHub](https://github.com/ce-rust/cerk/).

# This Component: MQTT Port using libmosquitto

This port publishes and/or subscribe CloudEvents to/from an MQTT topic.

The port is implemented with a fork of the [Mosquitto Client](https://docs.rs/mosquitto-client/0.1.5/mosquitto_client/)
and sends and receives messages according to the CloudEvents specification (see [MQTT Protocol Binding for CloudEvents v1.0](https://github.com/cloudevents/spec/blob/v1.0/mqtt-protocol-binding.md)).

The reason we are currently using a fork is that the version published on crates.io does currently not support the latest version of libmosquitto.
Because we proposed a [change to libmostquitto](https://github.com/eclipse/mosquitto/pull/1932) to better support our usecase we need to use the headerfiles of latest version of libmosquitto.
The goal is to used the published version at some point because otherwise this port can't be published on crates.io (see https://github.com/ce-rust/cerk/issues/88).


# Configurations

The configurations should be of type `cerk::kernel::Config::HashMap` and can have at the following fields:

## Required Fields

### host

The value has to by of type `Config::String` and contain a host name with protocol and port.

E.g. `Config::String(String::from("tcp://mqtt-broker:1883"))`

## Optional Fields

The following configurations are optional.

### send_topic

The value has to by of type `Config::String` and contain the MQTT topic name where the message will be sent to.

E.g. `Config::String(String::from("inbox"))`

### subscribe_topic

The value has to by of type `Config::String` and contain the MQTT topic which the router should subscribe to.

E.g. `Config::String(String::from("outbox"))`

### subscribe_qos

The value has to by of type `Config::U8` and contain the [quality of service](http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718099) for message delivery.

Currently, the following values are supported:

* 0: At most once delivery (default)
* 1: At least once delivery

E.g. `Config::U8(1)`

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
    ("send_topic".to_string(), Config::String("inbox".to_string())),
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
    ("subscribe_topic".to_string(), Config::String("outbox".to_string())),
    ("subscribe_qos".to_string(), Config::U8(1)),
]
.iter()
.cloned()
.collect();

let config = Config::HashMap(map);
```

### Full Configuration for receiving and sending events

```
use std::collections::HashMap;
use cerk::kernel::Config;

let map: HashMap<String, Config> = [
    ("host".to_string(), Config::String("tcp://mqtt-broker:1883".to_string())),
    ("send_topic".to_string(), Config::String("inbox".to_string())),
    ("subscribe_topic".to_string(), Config::String("outbox".to_string())),
    ("subscribe_qos".to_string(), Config::U8(1)),
]
.iter()
.cloned()
.collect();

let config = Config::HashMap(map);
```

*/

#![deny(missing_docs)]

#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate log;

mod port_mqtt;

pub use self::port_mqtt::{port_mqtt_mosquitto_start, PORT_MQTT_MOSQUITTO};
