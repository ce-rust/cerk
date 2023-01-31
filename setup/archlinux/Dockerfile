FROM --platform=linux/amd64 archlinux:base-devel-20230122.0.120412

RUN mkdir -p /opt/arch/var/lib/pacman
RUN pacman -Syyuu --debug
RUN pacman -Sy archlinux-keyring llvm clang gcc gcc-libs make cmake git glibc lib32-glibc --noconfirm --debug
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /cerk

CMD cargo test --all
