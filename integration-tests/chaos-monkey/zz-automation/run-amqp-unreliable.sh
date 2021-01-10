#!/bin/bash

mkdir -p output
output=./output/amqp-unreliable-$(date -u "+%Y%m%d%H%M%S").log
echo test started > $output

./setup-base.sh

kubectl apply -f ../continuous-run-config/ -f ../100k-messages-config/ -f ../100k-no-guarantee/ -f ../cerk-printer/ -f ../cerk-generator/ -f ../cerk/
kubectl rollout status deployments.apps/cerk-generator-deployment --timeout=1000s
echo "sequence_generator_started: $(date -u "+%Y%m%d%H%M%S")" >> $output

./wait-for-sequence-generator.sh
sleep 20

echo "starting with validator output: $(date -u "+%Y%m%d%H%M%S")" >> $output
kubectl logs deployments.apps/cerk-printer-deployment --tail 1000 | grep "cerk_port_dummies::port_sequence_validator" >> $output
echo "end: $(date -u "+%Y%m%d%H%M%S")" >> $output

echo test finished
