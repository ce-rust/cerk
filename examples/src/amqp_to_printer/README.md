# AMQP Example

Routes CloudEvents that are published on the RabbitMQ exchange to the the output port which prints the event to stdout.

## Run

1. run `docker-compose up`
2. run `cargo run --bin amqp_to_printer` IMPORTANT: you have to be in the directory `cerk/examples/src/amqp_to_printer/`
3. publish a CloudEvents on the test exchange <http://localhost:15672/#/exchanges/%2F/test> (login with `guest:guest`)
   e.g. `{"type":"test type","specversion":"1.0","source":"http://www.google.com","id":"id","contenttype":"application/json","data":"test"}`
4. observe CloudEvents arriving in the log output of the `amqp-router`
5. additionally you  could check the health with `curl -i localhost:3000`
