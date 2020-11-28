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

# This Component: HTTP Health Check Port

This component adds health check functionality to CERK via http.

It is registered as a port, but neither sends nor receives CloudEvents.

*/

#![deny(missing_docs)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

mod port_health_check;

pub use self::port_health_check::{port_health_check_http, PORT_HEALTH_CHECK_HTTP};
