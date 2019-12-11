# MQTT Example

Routes CloudEvents that are generated from an input port to an output port, the output port publishes the events on an MQTT Topic.
A second router subscribes to the same topic with an MQTT port and routs them to a port wich prints the event to stdout.

## Run

1. run `docker-compose up`
2. observe CloudEvents arriving in the log output of the `mqtt-client` docker-compose service
3. OPTIONAL: connect with an MQTT client (e.g. [MQTTBox for Chrome](https://chrome.google.com/webstore/detail/mqttbox/kaajoficamnjijhkeomgfljpicifbkaf)) to `mqtt://localhost:1883` and subscribe to topic `test`
