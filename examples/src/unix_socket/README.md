# UNIX Socket example

## requirements

* openbsd-netcat, command `nc`
  * **arch linux**: install it from the arch repository with `yaourt -S openbsd-netcat`
  * **mac**: install it with homebrew `brew install netcat`

## run

1. run `cargo run --bin unix_socket`
2. listen to the outgoing socket `nc -U ./cloud-events-out`
3. connect to the incomming socket `nc -U ./cloud-events-in`
4. send a CloudEvents over the `cloud-events-in` socket
