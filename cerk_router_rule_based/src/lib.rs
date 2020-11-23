/*!
This is a package for [CERK](https://github.com/ce-rust/cerk).
CERK is an open source [CloudEvents](https://github.com/cloudevents/spec) Router written in Rust with a MicroKernel architecture.
*/

#![deny(missing_docs)]

#[macro_use]
extern crate log;

mod router;
mod routing_rules;

pub use self::router::router_start;
pub use self::routing_rules::{CloudEventFields, RoutingRules, RoutingTable};
