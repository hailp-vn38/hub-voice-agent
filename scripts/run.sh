#!/usr/bin/env bash
# Build or run the hub app with the same shared target dir as scripts/test.sh.
# Usage: scripts/run.sh [build|run] [cargo args...]   e.g. scripts/run.sh run --release
# Default subcommand: run. Override the target dir with CARGO_TARGET_DIR.
set -euo pipefail

root=$(git rev-parse --show-toplevel)
common=$(cd "$root" && cd "$(git rev-parse --git-common-dir)/.." && pwd)
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$common/target}"

cmd="${1:-run}"
[ $# -gt 0 ] && shift

cd "$root"
exec cargo "$cmd" -p hub "$@"
