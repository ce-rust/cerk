From archlinux:base-20201108.0.8567

RUN pacman -Sy archlinux-keyring llvm clang gcc gcc-libs make cmake git --noconfirm
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /cerk

CMD cargo test --all
