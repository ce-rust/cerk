# CERK Router with UNIX Socket and MQTT for armv7

Routs CloudEvents that are generated from an input UNIX Socket port to a output UNIX Socket port and a MQTT output port.

## Requirement

This example require an armv7 based microprocessor.

## Run

1. build the docker image wich will be used to compile the binary
    `docker build . -t armv7`
2. build the binary (this command have to be executed in the root folder of the repository)
    `docker run -v ${PWD}/../../../:/cerk -ti armv7`
3. copy `target/armv7-unknown-linux-gnueabihf/release/hello_world`
