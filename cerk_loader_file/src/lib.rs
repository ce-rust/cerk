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
The configuration file could be passed by the env variable `$INIT_PATH` or just use the path `./init.json`.


### Example Config

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

#### Example ComponentStartLinks

```no_run
#[macro_use]
extern crate cerk_loader_file;
use cerk_loader_file::{start, ComponentStartLinks};

use cerk::runtime::{InternalServerId, InternalServerFn, InternalServerFnRefStatic, ScheduleFn, ScheduleFnRefStatic};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::kernel::{StartOptions, KernelFn};

fn dummy_scheduler(_: StartOptions, _: KernelFn) {}

fn dummy_router(_: InternalServerId, _: BoxedReceiver, _: BoxedSender) {}

fn dummy_config_loader(_: InternalServerId, _: BoxedReceiver, _: BoxedSender) {}

fn dummy_port(_: InternalServerId, _: BoxedReceiver, _: BoxedSender) {}

fn dummy_port_other(_: InternalServerId, _: BoxedReceiver, _: BoxedSender) {}

const SCHEDULER: ScheduleFnRefStatic = &(dummy_scheduler as ScheduleFn);
const ROUTER: InternalServerFnRefStatic = &(dummy_router as InternalServerFn);
const CONFIG_LOADER: InternalServerFnRefStatic = &(dummy_config_loader as InternalServerFn);
const PORT: InternalServerFnRefStatic = &(dummy_port as InternalServerFn);

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

## Examples

 * [AMQP to Printer](https://github.com/ce-rust/cerk/tree/master/examples/src/amqp_to_printer/)

*/

mod cerk_loader_file;
mod config_parser;
mod file_reader;
mod start_links;

#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

pub use self::cerk_loader_file::{load_by_path, start};
pub use self::start_links::ComponentStartLinks;
