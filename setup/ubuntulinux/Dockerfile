FROM ubuntu:20.04

ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=Europe/Zurich
RUN apt-get update
RUN apt-get install -y curl build-essential llvm clang gcc make cmake lsb-release libssl-dev wget libc6 \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup default 1.47.0

WORKDIR /cerk/

CMD cargo test --all
