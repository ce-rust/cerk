FROM lazzaretti/docker-rust-cerk:0.4.0 as build-env
WORKDIR /app
ADD . /app
RUN cargo build --release --locked

FROM gcr.io/distroless/cc
ENV RUST_LOG=info
COPY --from=build-env /app/target/release/cerk-common /app/init.json /app/config.json /
CMD ["./cerk-common"]
