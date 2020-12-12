# Common Docker Image for CERK 

[![Build status](https://badge.buildkite.com/4494e29d5f2c47e3fe998af46dff78a447800a76a68024e392.svg?branch=master)](https://buildkite.com/ce-rust/cerk)
[![Crates.io](https://img.shields.io/crates/v/cerk)](https://docs.rs/cerk/*/cerk/)
[![Docs status](https://docs.rs/cerk/badge.svg)](https://docs.rs/cerk/)
![Docker Cloud Build Status](https://img.shields.io/docker/cloud/build/cloudeventsrouter/cerk)

## Introduction

CERK lets you route your [CloudEvents](https://github.com/cloudevents/spec) between different different ports.
Ports are transport layer bindings over which CloudEvents can be exchanged.
It is built with modularity and portability in mind.


## Build & Run

Without any config changes hello world example will be executed.

1. `docker build . -t cerk-common`
2. `docker run cerk-common`

## Configure

Mount a custom `config.json` and `init.json` into the docker container.
