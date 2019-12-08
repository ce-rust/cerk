# Hello World Example

Routing CloudEvents that are generated from an input port to a output port, the output port print the result to the console.

## Requirements

* openbsd-netcat, command `nc`
  * **arch linux**: install it from the arch repository with `yaourt -S openbsd-netcat`
  * **mac**: install it with homebrew `brew install netcat`

## Run

### Docker

Run `docker-compose run hello-world`

### Local

Run `cargo run --bin hello_world`

## Output

In the logs you can see how the events are generated at the source and printed at the sink.

```text
2019-12-01T13:07:52Z INFO  hello_world] start hello world example
[2019-12-01T13:07:52Z INFO  cerk_runtime_threading::scheduler] start threading scheduler
[2019-12-01T13:07:52Z DEBUG cerk_runtime_threading::scheduler] schedule router thread
[2019-12-01T13:07:52Z DEBUG cerk_runtime_threading::scheduler] schedule config_loader thread
[2019-12-01T13:07:52Z INFO  cerk_router_broadcast::router] start broadcast router with id router
[2019-12-01T13:07:52Z DEBUG cerk_runtime_threading::scheduler] schedule dummy-sequence-generator thread
[2019-12-01T13:07:52Z INFO  hello_world] start static config loader with id config_loader
[2019-12-01T13:07:52Z DEBUG cerk_runtime_threading::scheduler] schedule dummy-logger-output thread
[2019-12-01T13:07:52Z INFO  cerk_port_dummies::port_sequence_generator] start sequence generator port with id dummy-sequence-generator
[2019-12-01T13:07:52Z INFO  cerk_port_dummies::port_printer] start printer port with id dummy-logger-output
[2019-12-01T13:07:52Z INFO  cerk_router_broadcast::router] router initiated
[2019-12-01T13:07:52Z INFO  cerk_port_dummies::port_printer] dummy-logger-output initiated
[2019-12-01T13:07:52Z DEBUG cerk_port_dummies::port_sequence_generator] send dummy event with sequence number 1 to kernel
[2019-12-01T13:07:52Z DEBUG cerk::kernel::kernel_start] received ConfigUpdated, forward to router
[2019-12-01T13:07:52Z DEBUG cerk::kernel::kernel_start] received ConfigUpdated, forward to dummy-sequence-generator
[2019-12-01T13:07:52Z DEBUG cerk::kernel::kernel_start] received ConfigUpdated, forward to dummy-logger-output
[2019-12-01T13:07:52Z INFO  cerk_port_dummies::port_printer] dummy-logger-output received ConfigUpdated
[2019-12-01T13:07:52Z DEBUG cerk::kernel::kernel_start] received OutgoingCloudEvent, forward to dummy-logger-output
[2019-12-01T13:07:52Z INFO  cerk_port_dummies::port_printer] dummy-logger-output received cloud event with id=1!
[2019-12-01T13:07:53Z DEBUG cerk_port_dummies::port_sequence_generator] send dummy event with sequence number 2 to kernel
[2019-12-01T13:07:53Z DEBUG cerk::kernel::kernel_start] received OutgoingCloudEvent, forward to dummy-logger-output
[2019-12-01T13:07:53Z INFO  cerk_port_dummies::port_printer] dummy-logger-output received cloud event with id=2!
```
