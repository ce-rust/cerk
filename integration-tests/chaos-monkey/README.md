# Chaos Monkey Test

## Requirements
You need a kubernetes cluster, e.g. a local minikube installation <https://minikube.sigs.k8s.io/docs/start/>
* on arch linux: 
    1. `yay -S minikube kubectl`
    2. `minikube start --driver virtualbox`

## Test

1. deploy all services
   `kubectl apply -f rabbitmq/ -f continuous-run-config/ -f cerk/ -f cerk-generator/ -f cerk-printer/`
2. observe the message flow at the reabbitmq
    1. do a prot forward to rabbitmq
        `kubectl port-forward statefulset/rabbitmq 15672`
    2. go to <http://localhost:15672>
        login with user `guest` pw `guest`
3. start to kill bods with the chaos monkey:
    1. deploy the chaos monkey
       `kubectl apply -f chaos-monkey/`
    2. observe the logs: 
        `kubectl logs --follow deployment/chaoskube`
4. Now we like to do a concrete test: we want to send 100'000 messages and look if all arrive
    1.  Delete the current sequence generator `kubectl delete deployments.apps cerk-generator-deployment`
    2.  Wait until all queues are empty or delete them over the [UI](http://localhost:15672)
    3.  Delete the printer `kubectl delete deployments.apps cerk-printer-deployment` and wait until the pod was deleted (`kubectl get pod -w`), too.
    4.  Deploy the new sequence generator and printer.
         The new sequence generator will only generate 10'000 messages and then stop.
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
    4. Redeploy the generator and printer `kubectl apply -f continuous-run-config/ -f 100k-messages-config/ -f cerk-printer/ -f cerk-generator/`
    5. Observe the result: `kubectl logs --follow deployment/cerk-printer-deployment` - probably it will not reach the end with the delivery of all 100k messages