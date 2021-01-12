#!/bin/bash

set -e

mkdir -p output
output=./output/mqtt-amqp-unreliable-$(date -u "+%Y%m%d%H%M%S").log
echo test started > $output

./setup-base.sh

kubectl apply -f ../continuous-run-config/ -f ../100k-no-guarantee/ -f ../100k-messages-config/ -f ../cerk-printer/ -f ../cerk-mqtt-amqp/
kubectl rollout status deployments.apps/cerk-printer-deployment --timeout=1000s
sleep 10 # wait for main router
kubectl apply -f ../100k-messages-config/ -f ../cerk-generator-mqtt/
kubectl rollout status deployments.apps/cerk-generator-deployment --timeout=1000s

kubectl get po >> "$output"

echo "sequence_generator_started: $(date -u "+%Y%m%d%H%M%S")" >> "$output"

./wait-for-sequence-generator.sh
sleep 20

echo "starting with validator output: $(date -u "+%Y%m%d%H%M%S")" >> "$output"
kubectl logs deployments.apps/cerk-printer-deployment --tail 1000 | grep cerk_port_dummies::port_sequence_validator >> "$output"
echo "end: $(date -u "+%Y%m%d%H%M%S")" >> "$output"

kubectl get po >> "$output"

./cleanup-base.sh >> "$output"

echo test finished
