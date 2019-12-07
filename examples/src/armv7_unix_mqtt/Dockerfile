FROM ubuntu

RUN apt-get update

RUN apt-get install -qq curl build-essential llvm clang gcc gcc-7-multilib gcc-arm-linux-gnueabihf make cmake lsb-release libssl-dev wget

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN printf "[target.armv7-unknown-linux-gnueabihf]\n\
linker = \"arm-linux-gnueabihf-gcc\"\n\
" > ~/.cargo/config


RUN rustup target add armv7-unknown-linux-gnueabihf

WORKDIR /cerk/

CMD cargo build --target armv7-unknown-linux-gnueabihf --release --bin hello_world
