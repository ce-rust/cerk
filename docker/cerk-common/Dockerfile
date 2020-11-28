FROM lazzaretti/docker-rust-cerk:0.3.0 as build-env
WORKDIR /app
ADD . /app
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=build-env /app/target/release/cerk-common /app/init.json /app/config.json /
CMD ["./cerk-common"]
