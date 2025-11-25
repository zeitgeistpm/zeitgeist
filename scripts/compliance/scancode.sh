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

require_cmd scancode

OUTPUT=${OUTPUT:-"$REPO_ROOT/compliance-report.json"}
DEFAULT_PROCESSES=$(command -v nproc >/dev/null 2>&1 && nproc || printf "4")
PROCESSES=${PROCESSES:-$DEFAULT_PROCESSES}

scancode --processes "$PROCESSES" \
  -clp "$REPO_ROOT" \
  --ignore "$REPO_ROOT/target" \
  --ignore "$REPO_ROOT/integration-tests/tmp" \
  --ignore "$REPO_ROOT/integration-tests/node_modules" \
  --ignore "**/target/**" \
  --ignore "*/node_modules/*" \
  --ignore "**/node_modules/**" \
  --ignore "**/integration-tests/node_modules/**" \
  --ignore "**/integration-tests/tmp/**" \
  --ignore "**/.venv/**" \
  --json-pp "$OUTPUT"
