# Rule Based Routing Example

CloudEvents that are generated from an input port are routed to an output port, 
but in this example only every thenth event gets routed to the output port because they are filterd by `id`.
The `id` has to end with `0`, thus only 10,20,30,... are printed.

## Run

### Docker

Run `docker-compose up`

### Local

Run `cargo run --bin rule_based_routing`


