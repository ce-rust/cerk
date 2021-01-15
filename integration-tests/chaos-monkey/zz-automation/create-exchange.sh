#!/bin/bash

set -e

rabbitmqadmin -V / declare exchange type=fanout name=input durable=false
rabbitmqadmin -V / declare exchange type=fanout name=output durable=false
