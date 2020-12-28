FROM ubuntu:18.04

RUN apt-get update
RUN apt-get install -y curl build-essential llvm clang gcc make cmake lsb-release libssl-dev wget git \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /cerk/

CMD cargo test --all
