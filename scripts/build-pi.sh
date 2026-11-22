#!/usr/bin/env bash
# Build the Raspberry Pi binary in a linux/arm64 Debian bookworm container.
# Output: target/pi/release/ai-hub
set -euo pipefail

cd "$(dirname "$0")/.."
IMAGE=ai-hub-pi-build
RUST_VERSION=$(sed -n 's/^channel *= *"\(.*\)"/\1/p' rust-toolchain.toml)

if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
    docker build --platform linux/arm64 \
        --build-arg "RUST_VERSION=$RUST_VERSION" \
        -f docker/pi-build.Dockerfile -t "$IMAGE" docker
fi

# The cargo registry lives in a named volume (the container's CARGO_HOME also
# holds the toolchain, so only the registry/git subdirs are mounted).
docker run --rm --platform linux/arm64 \
    -v "$PWD":/work \
    -v ai-hub-pi-cargo-registry:/usr/local/cargo/registry \
    -v ai-hub-pi-cargo-git:/usr/local/cargo/git \
    -e CARGO_TARGET_DIR=target/pi \
    "$IMAGE" \
    cargo build --release -p hub --no-default-features --features pi
