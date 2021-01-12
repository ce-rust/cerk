#!/bin/bash

if [ $(kubectl describe po mosquitto-0 rabbitmq-0 | grep Restart | awk '!/0/' | wc -l) -ne 0 ]; then
    echo "a broker crashed during the test!!!!"
    echo "will restart all brokers"
    kubectl delete po mosquitto-0 rabbitmq-0 --ignore-not-found --wait --force --grace-period 0
    sleep 60
    kubectl rollout status statefulsets.apps/rabbitmq --timeout=100s
    kubectl rollout status statefulsets.apps/mosquitto --timeout=100s
else
  echo "no broker restarts - ok"
fi
