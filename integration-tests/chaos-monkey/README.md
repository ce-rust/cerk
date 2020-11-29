# Chaos Monkey Test

0. You need a kubernetes cluster, e.g. a local minikube installation <https://minikube.sigs.k8s.io/docs/start/>
    * on arch linux: 
        1. `yay -S minikube kubectl`
        2. `minikube start --driver virtualbox`
1. deploy all services
   `kubectl apply -f rabbitmq/ -f cerk/ -f cerk-generator/ -f cerk-printer/`
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
