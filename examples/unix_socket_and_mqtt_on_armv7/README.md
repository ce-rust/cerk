# CERK Router with UNIX Socket and MQTT for armv7

Routes CloudEvents that are received on an input UNIX Socket port to an output UNIX Socket port and an MQTT output port.

## Requirement

This example require an armv7 based microprocessor.

## Run

1. Build the docker image which will be used to compile the binary:
    `docker build . -t armv7`
2. Build the binary
    `docker run -v ${PWD}/../../:/cerk -ti armv7`
3. Copy `target/armv7-unknown-linux-gnueabihf/release/unix_socket_and_mqtt_on_armv7` to the armv7 based microprocessor.
5. Start an MQTT broker
4. Start the router with the env variable `MQTT_BROKER_URL` which points to the broker, e.g. `MQTT_BROKER_URL=tpc://192.1.1.55:1883 ./unix_socket_and_mqtt_on_armv7`
