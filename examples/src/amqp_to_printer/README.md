# AMQP Example

Routes CloudEvents that are published on the RabbitMQ exchange to the the output port which prints the event to stdout.

## Run

1. run `docker-compose up`
2. run `cargo run --bin amqp_to_printer`
3. publish a CloudEvents on the test exchange <http://localhost:15672/#/exchanges/%2F/test> (login with `guest:guest`)
2. observe CloudEvents arriving in the log output of the `amqp-router`
