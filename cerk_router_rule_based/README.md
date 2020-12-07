# cerk_router_rule_based 0.2.6

[![Build status](https://badge.buildkite.com/4494e29d5f2c47e3fe998af46dff78a447800a76a68024e392.svg?branch=master)](https://buildkite.com/ce-rust/cerk)


This is a package for [CERK](https://github.com/ce-rust/cerk).
CERK is an open source [CloudEvents](https://github.com/cloudevents/spec) Router written in Rust with a MicroKernel architecture.

## Introduction

CERK lets you route your [CloudEvents](https://github.com/cloudevents/spec) between different different ports.
Ports are transport layer bindings over which CloudEvents can be exchanged.
It is built with modularity and portability in mind.

## Components

CERK comes with a couple of prefabricated components, but implementing custom components is easy.

A good overview is provided on [GitHub](https://github.com/ce-rust/cerk/).

## This Component: Rule Based Router

The rule-based router routes events based on the given configuration.

The configurations are structured in a tree format.
One configuration tree per output port needs to be configured.
The operations `And`, `Or`, `Contains`, `StartsWith` and more are supported.

## Configurations

The Socket expects a `Config::String` as configuration.
The string should be a json deserialized `routing_rules::RoutingTable`.

### Configuration Examples

#### Minimal

`Config::String("{}".to_string())`

#### Extended

```rust
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

## Examples

* [Rule Based Routing Example](https://github.com/ce-rust/cerk/tree/master/examples/src/rule_based_routing)



## Update Readme

The original readme text is a Rust doc comment in the [lib.rs](./src/lib.rs) file

1. `cargo install cargo-readme`
2. `cargo readme  > README.md`

## License

Apache-2.0
