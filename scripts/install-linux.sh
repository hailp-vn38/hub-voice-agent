#!/usr/bin/env bash
# Build and install the kiosk app on the Debian 13 (trixie) x86_64 tablet.
# Run on the tablet itself, from the repo checkout.
# Usage: scripts/install-linux.sh [--init]
# --init: first-time setup (packages, Rust, user, enable service).
set -euo pipefail

cd "$(dirname "$0")/.."
INIT=0
if [ "${1:-}" = "--init" ]; then INIT=1; fi

if [ "$INIT" = 1 ]; then
    sudo apt-get update
    sudo apt-get install -y build-essential curl pkg-config systemd-timesyncd \
        libudev-dev libinput-dev libdrm-dev libgbm-dev libegl-dev libgles-dev \
        libxkbcommon-dev libfontconfig-dev libseat-dev libasound2-dev libopus-dev
    command -v cargo >/dev/null 2>&1 \
        || curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
    id hub >/dev/null 2>&1 || sudo useradd --system --create-home --shell /usr/sbin/nologin hub
fi

# shellcheck disable=SC1091
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
CARGO_TARGET_DIR=target/kiosk cargo build --release -p hub --no-default-features --features kiosk

sudo install -D -m 755 target/kiosk/release/ai-hub /opt/ai-hub/ai-hub
sudo install -D -m 644 deploy/ai-hub.service /etc/systemd/system/ai-hub.service
sudo systemctl daemon-reload
if [ "$INIT" = 1 ]; then sudo systemctl enable ai-hub; fi
sudo systemctl restart ai-hub
