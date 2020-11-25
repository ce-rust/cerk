# cerk_loader_file 0.1.0

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

### This Component: File Base Loader

The cerk_loader_file link the different modules together and pass it to the `bootstrap` function.

It uses a `ComponentStartLinks` file with all links to the start functions and a configuration file.
The configuration file could be passed by the env variable `$CONFIG_PATH` or just use the path `./init.json`.


#### Example Config

```json
{
  "scheduler": "myschedulertype",
  "router": "myroutertype",
  "config_loader": "myconfig_loadertype",
  "ports": {
    "myport": "myporttype"
  }
}
```

##### Example ComponentStartLinks

```rust
 use cerk::runtime::{InternalServerId, InternalServerFn, ScheduleFn};
 use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
 use cerk::kernel::{StartOptions, KernelFn};
use cerk_loader_file::{start, ComponentStartLinks};

fn dummy_scheduler(_: StartOptions, _: KernelFn) {}

fn dummy_router(_: InternalServerId, _: BoxedReceiver, _: BoxedSender) {}

fn dummy_config_loader(_: InternalServerId, _: BoxedReceiver, _: BoxedSender) {}

fn dummy_port(_: InternalServerId, _: BoxedReceiver, _: BoxedSender) {}

fn dummy_port_other(_: InternalServerId, _: BoxedReceiver, _: BoxedSender) {}


let link = ComponentStartLinks {
        schedulers: [("myschedulertype".to_string(), &(dummy_scheduler as ScheduleFn))].iter()
            .cloned()
            .collect(),
        routers: [("myroutertype".to_string(), &(dummy_router as InternalServerFn))].iter()
            .cloned()
            .collect(),
        config_loaders: [("myconfig_loadertype".to_string(), &(dummy_config_loader as InternalServerFn))].iter()
            .cloned()
            .collect(),
        ports: [("myporttype".to_string(), &(dummy_port as InternalServerFn))].iter()
            .cloned()
            .collect(),
    };

start(link);
```


## Update Readme

The original readme text is a Rust doc comment in the [lib.rs](./src/lib.rs) file

1. `cargo install cargo-readme`
2. `cargo readme  > README.md`

## License

Apache-2.0
