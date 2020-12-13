# Reliable MQTT Integration Test

1. start test setup: `docker-compose up -d unlimited limited cerk`
  - run `docker-compose logs -f cerk` and wait until you see (the order may differ):
    ```
    DEBUG cerk_port_mqtt_mosquitto::port_mqtt] inbox_port connected: 0
    DEBUG cerk_port_mqtt_mosquitto::port_mqtt] rejecting_port connected: 0
    DEBUG cerk_port_mqtt_mosquitto::port_mqtt] outbox_port connected: 0
    ```
2. run tests: `docker-compose run test-executor`
3. reset test setup: `docker-compose down`