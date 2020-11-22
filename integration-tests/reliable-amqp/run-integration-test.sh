#!/bin/bash

set -e

echo "starting rabbitmq..."
docker-compose up -d rabbitmq
if (docker logs reliable-amqp_rabbitmq_1 -f 2>&1) | grep -m1 'Server startup complete'; then echo "rabbitmq ready"; fi;
echo "starting router..."
docker-compose up -d cerk-amqp
if (docker logs reliable-amqp_cerk-amqp_1 -f 2>&1) | grep -m1 'cerk_port_amqp::port_amqp] will consume'; then echo "router ready"; fi;
echo "starting tests..."
docker-compose up test-executor
