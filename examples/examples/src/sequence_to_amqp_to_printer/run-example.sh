#!/bin/bash

set -e

echo "starting rabbitmq..."
docker-compose up -d rabbitmq
if (docker logs sequence_to_amqp_to_printer_rabbitmq_1 -f 2>&1) | grep -m1 'Server startup complete'; then echo "rabbitmq ready"; fi;
echo "starting cerk-publisher..."
docker-compose up -d cerk-publisher
if (docker logs sequence_to_amqp_to_printer_cerk-publisher_1 -f 2>&1) | grep -m1 'send dummy event with sequence'; then echo "cerk-publisher is running"; fi;
echo "starting cerk-consumer..."
if (docker logs sequence_to_amqp_to_printer_cerk-consumer_1 -f 2>&1) | grep -m5 'dummy-logger-output received cloud event:'; then echo "cerk-consumer received 5 events"; fi;
