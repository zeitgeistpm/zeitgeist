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

require_cmd syft
require_cmd grype

SBOM_PATH=${SBOM_PATH:-"$REPO_ROOT/sbom.json"}

syft dir:"$REPO_ROOT" --exclude "./scripts/check-license/.venv" -o cyclonedx-json > "$SBOM_PATH"
grype sbom:"$SBOM_PATH" --fail-on medium --config "$REPO_ROOT/.grype.yaml"
