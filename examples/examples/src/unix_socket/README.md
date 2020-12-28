# UNIX Socket Example

Routes CloudEvents from an input UNIX Socket port to an output UNIX Socket port.

## Requirements

* openbsd-netcat, command `nc`
  * **arch linux**: install it from the arch repository with `yaourt -S openbsd-netcat`
  * **mac**: install it with homebrew `brew install netcat`

## Run

1. run `cargo run --bin unix_socket`
2. listen to the outgoing socket `nc -U ./cloud-events-out`
3. connect to the incomming socket `nc -U ./cloud-events-in`
4. send a CloudEvents over the `cloud-events-in` socket, 
    e.g. `{"type":"test type","specversion":"1.0","source":"http://www.google.com","id":"id","contenttype":"application/json","data":"test"}`
