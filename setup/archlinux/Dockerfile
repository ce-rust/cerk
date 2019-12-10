From archlinux:20191205

RUN pacman -Sy llvm clang gcc gcc-libs make cmake --noconfirm
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /cerk

CMD cargo test --all
