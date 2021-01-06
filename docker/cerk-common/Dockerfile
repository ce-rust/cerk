FROM lazzaretti/docker-rust-cerk:0.4.0 as build-env
WORKDIR /app

# add missing libraries, inspired by https://github.com/kyos0109/varnish60-distroless/blob/master/Dockerfile
RUN mkdir -p /opt/ && \
    cp -a --parents /lib/x86_64-linux-gnu/libgcc_s* /opt && \
    ls -al /opt/lib

ADD . /app
RUN cargo build --release --locked

FROM gcr.io/distroless/base-debian10
ENV RUST_LOG=info
COPY --from=build-env /opt /
COPY --from=build-env /app/target/release/cerk-common /app/init.json /app/config.json /
CMD ["./cerk-common"]
