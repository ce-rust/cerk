# cerk_loader_file 0.2.0

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
  "scheduler": "SCHEDULER",
  "router": "ROUTER",
  "config_loader": "CONFIG_LOADER",
  "ports": {
    "myport": "PORT"
  }
}
```

##### Example ComponentStartLinks

```rust
#[macro_use]
extern crate cerk_loader_file;
use cerk_loader_file::{start, ComponentStartLinks};

#
#
#
#
#
#

fn main() {
    let link = ComponentStartLinks {
            schedulers: fn_to_links![SCHEDULER],
            routers: fn_to_links![ROUTER],
            config_loaders: fn_to_links![CONFIG_LOADER],
            ports: fn_to_links![PORT],
        };

    start(link);
}
```

### Examples

 * [AMQP to Printer](https://github.com/ce-rust/cerk/tree/master/examples/src/amqp_to_printer/)


## Update Readme

The original readme text is a Rust doc comment in the [lib.rs](./src/lib.rs) file

1. `cargo install cargo-readme`
2. `cargo readme  > README.md`

## License

Apache-2.0
