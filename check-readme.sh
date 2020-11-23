#!/bin/bash

set -e

function check {
    diff <(cargo readme  -r $1) $1/README.md || (echo 1>&2 "Please update the $1/README with "'`'"cargo readme -r $1 > $1/README.md"'`' && exit 1 )
}

check cerk
check cerk_config_loader_file
check cerk_port_amqp
check cerk_port_dummies
check cerk_port_mqtt
check cerk_port_unix_socket
check cerk_router_broadcast
check cerk_router_rule_based
check cerk_runtime_threading
