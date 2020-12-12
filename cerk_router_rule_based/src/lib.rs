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

# This Component: Rule Based Router

The rule-based router routes events based on the given configuration.

The configurations are structured in a tree format.
One configuration tree per output port needs to be configured.
The operations `And`, `Or`, `Contains`, `StartsWith` and more are supported.

# Configurations

The Socket expects a `Config::String` as configuration.
The string should be a json deserialized `routing_rules::RoutingTable`.

## Configuration Examples

### Minimal

`Config::String("{}".to_string())`

### Extended

```
use serde_json;
use cerk_router_rule_based::{CloudEventFields, RoutingRules, RoutingTable};

let routing_rules: RoutingTable = [(
  "dummy-logger-output".to_string(),
  RoutingRules::And(vec![
    RoutingRules::Exact(
        CloudEventFields::Source,
        Some("dummy.sequence-generator".to_string()),
    ),
    RoutingRules::EndsWith(CloudEventFields::Id, "0".to_string()),
  ]),
)]
.iter()
.cloned()
.collect();

let routing_configs = serde_json::to_string(&routing_rules).unwrap();
```

# Examples

* [Rule Based Routing Example](https://github.com/ce-rust/cerk/tree/master/examples/src/rule_based_routing)


*/

#![deny(missing_docs)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

mod router;
mod routing_rules;

pub use self::router::{router_start, ROUTER_RULE_BASED};
pub use self::routing_rules::{CloudEventFields, RoutingRules, RoutingTable};
