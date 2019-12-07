# MQTT Example

## Run

1. run `docker-compose up`
2. observe CloudEvents arriving in the log output of the `mqtt-client` docker-compose service
3. OPTIONAL: connect with a MQTT client (e.g. [MQTTBox for Chrome](https://chrome.google.com/webstore/detail/mqttbox/kaajoficamnjijhkeomgfljpicifbkaf)) to `mqtt://localhost:1883` and subscribe to topic `test`
