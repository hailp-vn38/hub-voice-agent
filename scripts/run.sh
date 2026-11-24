#!/usr/bin/env bash
# Build or run the hub app with the same shared target dir as scripts/test.sh.
# Usage: scripts/run.sh [build|run] [cargo args...]   e.g. scripts/run.sh run --release
# Default subcommand: run. Override the target dir with CARGO_TARGET_DIR.
# The UI is laid out at 1920x1080; the default scale shows it at ~2560x1440 (3x the old 854x480 window).
# Override with SLINT_SCALE_FACTOR, e.g. SLINT_SCALE_FACTOR=0.444 scripts/run.sh for the old size.
set -euo pipefail

root=$(git rev-parse --show-toplevel)
common=$(cd "$root" && cd "$(git rev-parse --git-common-dir)/.." && pwd)
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$common/target}"
export SLINT_SCALE_FACTOR="${SLINT_SCALE_FACTOR:-1.332}"

cmd="${1:-run}"
[ $# -gt 0 ] && shift

cd "$root"
if [ -f .env ]; then set -a; . ./.env; set +a; fi
exec cargo "$cmd" -p hub "$@"
