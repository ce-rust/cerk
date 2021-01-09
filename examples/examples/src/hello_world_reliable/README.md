# Hello World Example

CloudEvents that are generated from an input port are routed to an output port, the output port print the result to the console.

## Run

### Docker

Run `docker-compose run hello-world-reliable`

### Local

Run `cargo run --bin hello_world_reliable`

## Output

In the logs you can see how the events are generated at the source and printed at the sink.

```text
2020-12-05T08:10:28Z INFO  hello_world_reliable] start hello world example
[2020-12-05T08:10:28Z INFO  cerk_runtime_threading::scheduler] start threading scheduler
[2020-12-05T08:10:28Z DEBUG cerk_runtime_threading::scheduler] schedule router thread
[2020-12-05T08:10:28Z DEBUG cerk_runtime_threading::scheduler] schedule config_loader thread
[2020-12-05T08:10:28Z INFO  cerk_router_broadcast::router] start broadcast router with id router
[2020-12-05T08:10:28Z INFO  hello_world_reliable] start static config loader with id config_loader
[2020-12-05T08:10:28Z DEBUG cerk_runtime_threading::scheduler] schedule dummy-sequence-generator thread
[2020-12-05T08:10:28Z DEBUG cerk_runtime_threading::scheduler] schedule dummy-validator-output thread
[2020-12-05T08:10:28Z INFO  cerk_port_dummies::port_sequence_generator] start sequence generator port with id dummy-sequence-generator
[2020-12-05T08:10:28Z INFO  cerk_port_dummies::port_sequence_validator] start sequence validator port with id dummy-validator-output
[2020-12-05T08:10:28Z INFO  cerk_router_broadcast::router] router initiated
[2020-12-05T08:10:28Z DEBUG cerk::kernel::kernel_start] received ConfigUpdated, forward to router
[2020-12-05T08:10:28Z DEBUG cerk::kernel::kernel_start] received ConfigUpdated, forward to dummy-sequence-generator
[2020-12-05T08:10:28Z DEBUG cerk::kernel::kernel_start] received ConfigUpdated, forward to dummy-validator-output
[2020-12-05T08:10:28Z INFO  cerk_port_dummies::port_sequence_validator] dummy-validator-output received ConfigUpdated
[2020-12-05T08:10:28Z INFO  cerk_port_dummies::port_sequence_generator] dummy-sequence-generator start generating events
[2020-12-05T08:10:28Z DEBUG cerk_port_dummies::port_sequence_generator] send dummy event with sequence number 1 to kernel
[2020-12-05T08:10:28Z DEBUG cerk::kernel::kernel_start] received RoutingResult for event_id=1
[2020-12-05T08:10:28Z DEBUG cerk::kernel::kernel_start] all routing sent for event_id=1
[2020-12-05T08:10:28Z DEBUG cerk_port_dummies::port_sequence_validator] event 1 was received
[2020-12-05T08:10:28Z DEBUG cerk_port_dummies::port_sequence_validator] event 1 delay from generation to validator 0ms
[2020-12-05T08:10:28Z INFO  cerk_port_dummies::port_sequence_validator] missing events: 49
[2020-12-05T08:10:28Z DEBUG cerk::kernel::kernel_start] received OutgoingCloudEventProcessed from=dummy-validator-output event_id=1
[2020-12-05T08:10:28Z DEBUG cerk::kernel::kernel_start] delivery for event_id=1 was successful (all out port processing were successful) -> ack to sender
[2020-12-05T08:10:29Z DEBUG cerk_port_dummies::port_sequence_generator] send dummy event with sequence number 2 to kernel
[2020-12-05T08:10:29Z DEBUG cerk::kernel::kernel_start] received RoutingResult for event_id=2
[2020-12-05T08:10:29Z DEBUG cerk::kernel::kernel_start] all routing sent for event_id=2
[2020-12-05T08:10:29Z DEBUG cerk_port_dummies::port_sequence_validator] event 2 was received
[2020-12-05T08:10:29Z DEBUG cerk_port_dummies::port_sequence_validator] event 2 delay from generation to validator 0ms
```
