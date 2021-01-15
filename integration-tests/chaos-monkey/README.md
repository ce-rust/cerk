# Chaos Monkey Test

## Requirements

You need a kubernetes cluster, e.g. a local minikube installation <https://minikube.sigs.k8s.io/docs/start/>
* on arch linux: 
    1. `yay -S minikube kubectl`
    2. `minikube start` (delete the cluster at the end with `minikube delete`)

This test routes 100'000 messages is therefore quite resource intensive.
The test was executed with a minikube cluster with 16 Cores and 32Gb memory (`minikube start --cpus=16 --memory=32G`)

## Test AMQP to AMQP

In this test we route messages from one RabbitMQ to another one.

1. deploy all services
   `kubectl apply -f rabbitmq/ -f continuous-run-config/ -f cerk/ -f cerk-generator/ -f cerk-printer/`
2. observe the message flow at the RabbitMQ
    1. do a port forward to RabbitMQ
        `kubectl port-forward statefulset/rabbitmq 15672`
    2. go to <http://localhost:15672>
        login with user `guest` pw `guest`
3. start to kill pods with the chaos monkey:
    1. deploy the chaos monkey
       `kubectl apply -f chaos-monkey/`
    2. observe the logs: 
        `kubectl logs --follow deployment/chaoskube`
4. Now we like to do a concrete test: we want to send 100'000 messages and look if all arrive
    1.  Delete the current sequence generator `kubectl delete deployments.apps cerk-generator-deployment`
    2.  Wait until all queues are empty or delete them over the [UI](http://localhost:15672)
    3.  Delete the printer `kubectl delete deployments.apps cerk-printer-deployment` and wait until the pod was deleted (`kubectl get pod -w`), too.
    4.  Deploy the new sequence generator and printer.
         The new sequence generator will only generate 100'000 messages and then stop.
         The new printer will verify that all 100'000 messages are delivered At Least Once.
        `kubectl apply -f continuous-run-config/ -f 100k-messages-config/ -f cerk-printer/ -f cerk-generator/` 
    5.   Observe the result: `kubectl logs --follow deployment/cerk-printer-deployment` - it should reach the end with:
        ```
       [2020-11-30T14:34:44Z INFO  cerk_port_dummies::port_sequence_validator] missing events: 0
       [2020-11-30T14:34:44Z INFO  cerk_port_dummies::port_sequence_validator] **************************************************************************
       [2020-11-30T14:34:44Z INFO  cerk_port_dummies::port_sequence_validator] ************************** received all events! **************************
       [2020-11-30T14:34:44Z INFO  cerk_port_dummies::port_sequence_validator] *************************************************************************
       ```
5. Now we want to see the difference: we run the 100'000 messages without the delivery guarantee on the main router (`cerk-deployment`)
    1. Delete the routers `kubectl delete deployments.apps cerk-deployment`
    2. Deploy the router with no delivery guarantee `kubectl apply -f continuous-run-config/ -f 100k-no-guarantee/ -f cerk/`
    3. Delete the generator and printer `kubectl delete deployments.apps cerk-generator-deployment cerk-printer-deployment` and wait until the pods were deleted (`kubectl get pod -w`), too. 
    4. Redeploy the generator and printer `kubectl apply -f continuous-run-config/ -f 100k-no-guarantee/ -f 100k-messages-config/ -f cerk-printer/ -f cerk-generator/`
    5. Observe the result: `kubectl logs --follow deployment/cerk-printer-deployment` - probably it will not reach the end with the delivery of all 100k messages
6. clean the cluster `kubectl delete all --all`

## Test AMQP to AMQP & MQTT

Test setup: (generator ->amqp) -> (amqp -> mqtt) -> (mqtt -> validator)

1. Start the test that routs 100'000 messages reliable
    1. `kubectl apply -f continuous-run-config/ -f rabbitmq/ -f mosquitto/ -f chaos-monkey/` and wait until the services are up and running (`kubernetes get po -w`)
    2. create an exchange `input` as type `fanout` with durability `transient` (in the previews example this was created with the continuous run at the beginning, here we do it manually)
    2. `kubectl apply -f continuous-run-config/ -f 100k-messages-config/ -f cerk-printer/  -f cerk-printer-mqtt/ -f 100k-messages-config/ -f cerk-amqp-mqtt/` and wait until the queues `output` and `input` on the RabbitMQ were created (<http://localhost:15672> when RabbitMQ is forwarded)
    3. `kubectl apply -f 100k-messages-config/ -f cerk-generator/`
    4. observe the logs:
        * Observe the result of the amqp validator: `kubectl logs --follow deployment/cerk-printer-deployment` - it should reach the end with:
              ```
             [2020-11-30T14:34:44Z INFO  cerk_port_dummies::port_sequence_validator] missing events: 0
             [2020-11-30T14:34:44Z INFO  cerk_port_dummies::port_sequence_validator] **************************************************************************
             [2020-11-30T14:34:44Z INFO  cerk_port_dummies::port_sequence_validator] ************************** received all events! **************************
             [2020-11-30T14:34:44Z INFO  cerk_port_dummies::port_sequence_validator] *************************************************************************
             ```
        * Observe the result of the mqtt validator: `kubectl logs --follow deployment/cerk-printer-mqtt-deployment` - it should reach the end with:
              ```
             [2020-11-30T14:34:44Z INFO  cerk_port_dummies::port_sequence_validator] missing events: 0
             [2020-11-30T14:34:44Z INFO  cerk_port_dummies::port_sequence_validator] **************************************************************************
             [2020-11-30T14:34:44Z INFO  cerk_port_dummies::port_sequence_validator] ************************** received all events! **************************
             [2020-11-30T14:34:44Z INFO  cerk_port_dummies::port_sequence_validator] *************************************************************************
             ```
2. Start the test that routs 100'000 messages un-reliable
    1. Delete the generator, printer, and reliable router `kubectl delete deployments.apps cerk-generator-deployment cerk-printer-deployment cerk-printer-mqtt-deployment cerk-deployment` and wait until the pods were deleted (`kubectl get pod -w`), too
    2. check that all queues are empty
    3. `kubectl apply -f 100k-no-guarantee/ -f 100k-messages-config/ -f cerk-amqp-mqtt/`
    4. `kubectl apply -f continuous-run-config/ -f 100k-messages-config/ -f cerk-printer/ -f cerk-printer-mqtt/ -f cerk-generator/`
    5. observe the logs: this time they probably do not end with `missing events: 0`
3. clean the cluster `kubectl delete all --all`

## Test MQTT to AMQP

Test setup: (generator ->mqtt) -> (mqtt -> amqp) -> (amqp -> validator)

1. Start the test that routs 100'000 messages reliable
    1. `kubectl apply -f continuous-run-config/ -f rabbitmq/ -f mosquitto/ -f chaos-monkey/` and wait until the services are up and running (`kubernetes get po -w`)
    2. `kubectl apply -f continuous-run-config/ -f 100k-messages-config/ -f cerk-printer/ -f 100k-messages-config/ -f cerk-mqtt-amqp/`
    3. `kubectl apply -f 100k-messages-config/ -f cerk-generator-mqtt/`
    4. observe the logs
2. Start the test that routes 100'000 messages unreliably
    1. Delete the generator, printer, and reliable router `kubectl delete deployments.apps cerk-generator-deployment cerk-printer-deployment` and wait until the pods were deleted (`kubectl get pod -w`), too
    2. check that all queues are empty
    3. `kubectl apply -f 100k-no-guarantee/ -f 100k-messages-config/ -f cerk-mqtt-amqp/`
    4. `kubectl apply -f continuous-run-config/ -f 100k-messages-config/ -f cerk-printer/ -f cerk-generator-mqtt/`
    5. observe the logs: this time they probably do not end with `missing events: 0`
2. clean the cluster `kubectl delete all --all`

## Automated tests

All listed tests and more are automated.
They can be executed with `cd zz-automation; ./run-all-in-loop.sh`

Additional requirement:

* `rabbitmqadmin` (with yay: `yay -S rabbitmqadmin`)
