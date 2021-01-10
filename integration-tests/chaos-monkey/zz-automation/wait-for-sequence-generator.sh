#!/bin/bash

while true; do
  if kubectl logs deployments.apps/cerk-generator-deployment --tail 100000 | grep -q "delivery for event_id=100000 was successful"; then
    echo "generator finished"
    exit 0;
  fi;
  echo "wait for finished generator..."
  sleep 5
done
