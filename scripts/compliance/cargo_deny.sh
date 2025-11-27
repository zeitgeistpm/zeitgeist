#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -euo pipefail

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$REPO_ROOT"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command '$1'. Install it and retry." >&2
    exit 127
  fi
}

require_cmd cargo
require_cmd cargo-deny

LOG_DIR=${CARGO_DENY_LOG_DIR:-"$REPO_ROOT/scripts/compliance/logs"}
mkdir -p "$LOG_DIR"

LOG_PATH=${CARGO_DENY_LOG:-"$LOG_DIR/cargo-deny.log"}
ERR_LOG_PATH=${CARGO_DENY_ERR_LOG:-"$LOG_DIR/cargo-deny-errors.log"}
CARGO_DENY_HOME=${CARGO_DENY_HOME:-"$REPO_ROOT/.cargo-deny"}
export CARGO_DENY_HOME
mkdir -p "$CARGO_DENY_HOME"
ADVISORY_DB_PATH=${CARGO_DENY_ADVISORY_DB_PATH:-"$REPO_ROOT/.cargo-deny/advisory-dbs"}
mkdir -p "$ADVISORY_DB_PATH"

DISABLE_FETCH_FLAG=()
if [[ -n "${CARGO_DENY_DISABLE_FETCH:-}" ]]; then
  DISABLE_FETCH_FLAG+=(--disable-fetch)
fi

: > "$LOG_PATH"
: > "$ERR_LOG_PATH"

run_check() {
  local check="$1"
  echo "Running: cargo deny check $check"
  cargo deny check "$check" "${DISABLE_FETCH_FLAG[@]}" >>"$LOG_PATH" 2>&1
}

run_check licenses
run_check bans
run_check sources
run_check advisories

grep -nEi 'error|warn' "$LOG_PATH" > "$ERR_LOG_PATH" || true

echo
echo "Full log:    $LOG_PATH"
echo "Errors log:  $ERR_LOG_PATH"
if [[ -s "$ERR_LOG_PATH" ]]; then
  echo "Found errors/warnings. View with: less -R $ERR_LOG_PATH"
else
  echo "No errors/warnings recorded (errors log is empty)."
fi
echo "View full log with: less -R $LOG_PATH"
