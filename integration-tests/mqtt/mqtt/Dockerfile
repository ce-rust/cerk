FROM lazzaretti/docker-rust-cerk:0.3.0 as build-env
WORKDIR /app/integration-tests/mqtt/mqtt
ADD ./integration-tests/mqtt/mqtt /app/integration-tests/mqtt/mqtt
ADD ./cerk_port_mqtt_mosquitto/ /app/cerk_port_mqtt_mosquitto
ADD ./integration-tests/mqtt/mqtt/libmosquitto.so.1 /usr/local/lib/
RUN ln -sf /usr/local/lib/libmosquitto.so.1 /usr/local/lib/libmosquitto.so
RUN ln -sf /usr/local/lib/libmosquitto.so.1 /usr/lib/libmosquitto.so.1 
RUN ln -sf /usr/local/lib/libmosquitto.so.1 /usr/lib/libmosquitto.so
RUN cargo build --release --locked

ENTRYPOINT cargo run

#FROM eclipse-mosquitto
#ENV RUST_LOG=info
#COPY --from=build-env /app/integration-tests/mqtt/mqtt/target/release/mqtt /app/integration-tests/mqtt/mqtt/init.json /app/integration-tests/mqtt/mqtt/config.json /
#CMD ["./mqtt"]
