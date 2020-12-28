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

# This Component: File Based Config Loader

This port loads configurations from a json file.

The file path could be set with the env variable `CONFIG_PATH`, default is `./config.json`.

## Example Config

```json
{
  "routing_rules": [
    "dummy-logger-output"
  ],
  "ports": {
    "ampq-input": {
      "uri": "amqp://127.0.0.1:5672/%2f",
      "consume_channels": [
        {
          "name": "test",
          "ensure_queue": true,
          "bind_to_exchange": "test"
        }
      ],
      "publish_channels": [
        {
          "name": "test",
          "ensure_exchange": true
        }
      ]
    },
    "dummy-logger-output": null
  }
}
```

## Examples

 * [AMQP to Printer](https://github.com/ce-rust/cerk/tree/master/examples/examples/src/amqp_to_printer/)

*/

#![deny(missing_docs)]

#[macro_use]
extern crate log;

mod config_loader_file;
mod config_parser;
mod file_reader;

pub use self::config_loader_file::{config_loader_file_start, CONFIG_LOADER_FILE};
