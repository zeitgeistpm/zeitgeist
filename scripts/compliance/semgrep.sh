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

CONFIG="$REPO_ROOT/.github/semgrep-rules.yml"

if [[ ! -f "$CONFIG" ]]; then
  echo "Semgrep config not found at $CONFIG" >&2
  exit 1
fi

require_cmd semgrep

semgrep --config "$CONFIG"
