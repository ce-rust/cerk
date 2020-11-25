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

## This Component: File Base Loader

The cerk_loader_file link the different modules together and pass it to the `bootstrap` function.

It uses a `ComponentStartLinks` file with all links to the start functions and a configuration file.
The configuration file could be passed by the env variable `$CONFIG_PATH` or just use the path `./init.json`.


### Example Config

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

#### Example ComponentStartLinks

```no_run
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

*/

mod cerk_loader_file;
mod config_parser;
mod file_reader;
mod start_links;

#[macro_use]
extern crate log;

pub use self::cerk_loader_file::{load_by_path, start};
pub use self::start_links::ComponentStartLinks;
