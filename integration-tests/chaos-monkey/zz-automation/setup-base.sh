#!/bin/bash

set -e

# with this command we could be sure that the pod is deleted
kubectl scale deployment cerk-printer-deployment --replicas 0 --timeout 1000s || true
kubectl scale deployment cerk-printer-mqtt-deployment --replicas 0 --timeout 1000s || true
kubectl scale deployment cerk-deployment --replicas 0 --timeout 1000s || true
kubectl scale deployment cerk-generator-deployment --replicas 0 --timeout 1000s || true

kubectl delete deployments cerk-deployment cerk-generator-deployment cerk-printer-deployment cerk-printer-mqtt-deployment --ignore-not-found --wait --force --grace-period 0

kubectl apply -f ../continuous-run-config/ -f ../rabbitmq/ -f ../mosquitto/ -f ../chaos-monkey/
kubectl rollout status statefulsets.apps/rabbitmq --timeout=100s
kubectl rollout status statefulsets.apps/mosquitto --timeout=100s

# verify that only the new deployed components are here
while [ $(kubectl get po | wc -l) -ne 4 ] || [ $(kubectl get po | grep "chaoskube" | wc -l) -ne 1 ] || [ $(kubectl get po | grep "rabbitmq" | wc -l) -ne 1  ] || [ $(kubectl get po | grep "mosquitto" | wc -l) -ne 1 ]; do
    echo "wait for matching deployed..."
    sleep 1
done

fuser -k 15672/tcp || true # kill process on port
kubectl port-forward statefulset/rabbitmq 15672 || true &\
while ! curl 127.0.0.1:15672 > /dev/null 2> /dev/null; do echo 'wait for port-forward...'; sleep 10; done
./create-exchange.sh
