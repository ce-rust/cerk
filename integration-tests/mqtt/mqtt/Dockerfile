FROM lazzaretti/docker-rust-cerk:0.3.0 as build-env
WORKDIR /app
ADD . /app
RUN cargo build --release --locked

FROM eclipse-mosquitto
ENV RUST_LOG=info
COPY --from=build-env /app/target/release/mqtt /app/init.json /app/config.json /
CMD ["./mqtt"]
