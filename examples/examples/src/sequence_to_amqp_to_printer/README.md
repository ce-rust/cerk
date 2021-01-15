# AMQP Example

Routes CloudEvents that are published on the RabbitMQ exchange to the the output port which prints the event to stdout.

## Run

1. start RabbitMQ `docker-compose up -d rabbitmq`
2. start the publisher `docker-compose up -d cerk-publisher`
3. start the consumer `docker-compose up -d cerk-consumer`
4. observe the output routing on the queue <http://localhost:15672/#/queues/%2F/test> and in the docker-compose logs
5. stop the example `docker-compose down`
