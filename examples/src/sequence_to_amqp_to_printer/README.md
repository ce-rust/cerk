# AMQP Example

Routes CloudEvents that are published on the RabbitMQ exchange to the the output port which prints the event to stdout.

## Run

1. run `docker-compose up`
3. observe the output routing on the queue <http://localhost:15672/#/queues/%2F/test> and in the docker-compose logs
