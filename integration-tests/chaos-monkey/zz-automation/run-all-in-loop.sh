#!/bin/bash

set -e

for (( i=1; i>0; i++ ));do
  echo "***************************"
  echo "running iteration ${i}"
  echo "***************************"
  echo "running amqp-mqtt-reliable ***************************"
  ./run-amqp-mqtt-reliable.sh
  echo "running amqp-mqtt-unreliable ***************************"
  ./run-amqp-mqtt-unreliable.sh
  echo "running amqp-reliable ***************************"
  ./run-amqp-reliable.sh
  echo "running amqp-unreliable ***************************"
  ./run-amqp-unreliable.sh
  echo "running mqtt-amqp-reliable ***************************"
  ./run-mqtt-amqp-reliable.sh
  echo "running mqtt-amqp-unreliable ***************************"
  ./run-mqtt-amqp-unreliable.sh
  echo "running mqtt-reliable ***************************"
  ./run-mqtt-reliable.sh
  echo "running mqtt-unreliable ***************************"
  ./run-mqtt-unreliable.sh
done
