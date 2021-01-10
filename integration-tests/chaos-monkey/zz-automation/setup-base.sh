#!/bin/bash

# with this command we could be sure that the pod is deleted
kubectl scale deployment cerk-printer-deployment --replicas 0 --timeout 1000s || true
kubectl scale deployment cerk-printer-mqtt-deployment --replicas 0 --timeout 1000s || true
kubectl scale deployment cerk-deployment --replicas 0 --timeout 1000s || true
kubectl scale deployment cerk-generator-deployment --replicas 0 --timeout 1000s || true

kubectl delete deployments cerk-deployment cerk-generator-deployment cerk-printer-deployment cerk-printer-mqtt-deployment --ignore-not-found --wait --force --grace-period 0

kubectl apply -f ../continuous-run-config/ -f ../rabbitmq/ -f ../mosquitto/ -f ../chaos-monkey/
kubectl rollout status statefulsets.apps/rabbitmq --timeout=100s
kubectl rollout status statefulsets.apps/mosquitto --timeout=100s
kubectl port-forward statefulset/rabbitmq 15672 || true &
./create-exchange.sh
