#!/usr/bin/env bash
# Usage: scripts/deploy-pi.sh [--init] <host>
# --init: first-time setup (packages, user, enable service).
set -euo pipefail

cd "$(dirname "$0")/.."
INIT=0
if [ "${1:-}" = "--init" ]; then INIT=1; shift; fi
HOST=${1:?usage: deploy-pi.sh [--init] <host>}
BIN=target/pi/release/ai-hub

[ -f "$BIN" ] || { echo "missing $BIN; run scripts/build-pi.sh first" >&2; exit 1; }

if [ "$INIT" = 1 ]; then
    ssh "$HOST" 'sudo apt-get update && sudo apt-get install -y libinput10 libgbm1 libegl1 libgles2 libfontconfig1 libseat1 rsync \
        && (id hub >/dev/null 2>&1 || sudo useradd --system --create-home --shell /usr/sbin/nologin hub) \
        && sudo mkdir -p /opt/ai-hub'
fi

rsync -av --rsync-path="sudo rsync" "$BIN" "$HOST:/opt/ai-hub/ai-hub"
rsync -av --rsync-path="sudo rsync" deploy/ai-hub.service "$HOST:/etc/systemd/system/ai-hub.service"
ssh "$HOST" 'sudo systemctl daemon-reload'
if [ "$INIT" = 1 ]; then
    ssh "$HOST" 'sudo systemctl enable ai-hub'
fi
ssh "$HOST" 'sudo systemctl restart ai-hub'
