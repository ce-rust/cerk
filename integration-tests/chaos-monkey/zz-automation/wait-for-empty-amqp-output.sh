#!/bin/bash

set -e

while ! rabbitmqadmin -V / list queues | grep -q "output     | 0"; do
  echo "wait for empty queue..."
  sleep 5
done

echo "queue is empty"
