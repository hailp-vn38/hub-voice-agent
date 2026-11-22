#!/usr/bin/env bash
# Token-lean test runner for agents.
# Usage: scripts/test.sh [cargo test args...]   e.g. scripts/test.sh -p hub-core
# Success: prints one summary line. Failure: prints only failures/errors; full log path at the end.
#
# All checkouts and git worktrees share one target dir (the main checkout's target/),
# so dependencies are built once. Override with CARGO_TARGET_DIR.
set -uo pipefail

root=$(git rev-parse --show-toplevel)
common=$(cd "$root" && cd "$(git rev-parse --git-common-dir)/.." && pwd)
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$common/target}"
export CARGO_TERM_COLOR=never

log="$CARGO_TARGET_DIR/test-last.log"
mkdir -p "$CARGO_TARGET_DIR"

cd "$root"
cargo test --workspace --quiet "$@" >"$log" 2>&1
status=$?

passed=$(grep -oE '[0-9]+ passed' "$log" | awk '{s+=$1} END {print s+0}')
failed=$(grep -oE '[0-9]+ failed' "$log" | awk '{s+=$1} END {print s+0}')

if [ $status -eq 0 ]; then
    echo "ok: $passed passed"
    exit 0
fi

# Compile errors (capped), then test failure details.
grep -A12 -E '^error(\[E[0-9]+\])?:' "$log" | head -80
sed -n '/^failures:$/,/^test result:/p' "$log" | head -120
echo "FAIL: $passed passed, $failed failed (exit $status). Full log: $log"
exit $status
