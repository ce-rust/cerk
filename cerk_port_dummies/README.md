# cerk_port_dummies

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

## This Crate: Dummy Ports

This crate contains some dummy ports for testing and demonstrations.

The ports are:

* port_printer
* port_sequence_generator
* port_sequence_validator


## Update Readme

The original readme text is a Rust doc comment in the [lib.rs](./src/lib.rs) file

1. `cargo install cargo-readme`
2. `cargo readme  > README.md`

## License

Apache-2.0
