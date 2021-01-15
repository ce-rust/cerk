#!/bin/bash

set -e

for (( i=1; i>0; i++ ));do
  echo "***************************"
  echo "running iteration ${i}"
  echo "***************************"
  echo "running amqp-reliable ***************************"
  ./run-amqp-reliable.sh
  echo "running amqp-unreliable ***************************"
  ./run-amqp-unreliable.sh
done
