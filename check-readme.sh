#!/bin/bash

set -e

param=$1

function check {
  if [ "$param" == "update" ]; then
    cargo readme -r $1 > $1/README.md
  else
    diff <(cargo readme  -r $1) $1/README.md || (echo 1>&2 "Please update the $1/README with "'`'"cargo readme -r $1 > $1/README.md"'`' && exit 1 )
  fi
}

check cerk
check cerk_config_loader_file
check cerk_loader_file
check cerk_port_amqp
check cerk_port_dummies
check cerk_port_health_check_http
check cerk_port_mqtt
check cerk_port_unix_socket
check cerk_router_broadcast
check cerk_router_rule_based
check cerk_runtime_threading
