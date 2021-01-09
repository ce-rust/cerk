#! /bin/sh

(cd cerk; cargo publish)
(cd cerk_port_amqp; cargo publish)
(cd cerk_port_dummies; cargo publish)
(cd cerk_port_health_check_http; cargo publish)
(cd cerk_port_mqtt; cargo publish)
(cd cerk_port_mqtt_mosquitto; cargo publish)
(cd cerk_port_unix_socket; cargo publish)
(cd cerk_router_rule_based; cargo publish)
(cd cerk_runtime_threading; cargo publish)
