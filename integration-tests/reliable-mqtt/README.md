# Reliable MQTT Integration Test

This test uses `cerk_port_mqtt_mosquitto` with a patched version of libmosquitto (check out the ports [README](https://github.com/ce-rust/cerk/tree/master/cerk_port_mqtt_mosquitto) for more details).

## Rum Tests

1. start test setup: `docker-compose up -d unlimited limited cerk`
  - run `docker-compose logs -f cerk` and wait until you see (the order may differ):
    ```
    DEBUG cerk_port_mqtt_mosquitto::port_mqtt] inbox_port connected: 0
    DEBUG cerk_port_mqtt_mosquitto::port_mqtt] rejecting_port connected: 0
    DEBUG cerk_port_mqtt_mosquitto::port_mqtt] outbox_port connected: 0
    ```
2. run tests: `docker-compose run test-executor`
3. reset test setup: `docker-compose down`