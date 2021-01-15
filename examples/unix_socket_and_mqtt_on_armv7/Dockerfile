FROM ubuntu:18.04

RUN apt-get update
RUN apt-get install -y curl build-essential llvm clang gcc gcc-7-multilib make cmake lsb-release libssl-dev wget \
    && apt-get install -qq gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN printf "[target.armv7-unknown-linux-gnueabihf]\n\
linker = \"arm-linux-gnueabihf-gcc\"\n\
" > ~/.cargo/config

RUN rustup target add armv7-unknown-linux-gnueabihf

WORKDIR /cerk/examples/unix_socket_and_mqtt_on_armv7

CMD cargo build --target armv7-unknown-linux-gnueabihf --release
