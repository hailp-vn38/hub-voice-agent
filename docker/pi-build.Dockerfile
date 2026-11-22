FROM --platform=linux/arm64 debian:bookworm

ARG RUST_VERSION=1.98

RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential ca-certificates curl git \
        libudev-dev libinput-dev libdrm-dev libgbm-dev libegl-dev libgles-dev \
        libxkbcommon-dev libfontconfig-dev libseat-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/usr/local/rustup CARGO_HOME=/usr/local/cargo PATH=/usr/local/cargo/bin:$PATH
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain ${RUST_VERSION}

WORKDIR /work
