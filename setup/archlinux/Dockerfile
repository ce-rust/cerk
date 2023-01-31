FROM --platform=linux/amd64 archlinux:base-devel-20230129.0.122153

RUN pacman -Sy archlinux-keyring llvm clang gcc gcc-libs make cmake git glibc lib32-glibc --noconfirm
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /cerk

CMD cargo test --all
