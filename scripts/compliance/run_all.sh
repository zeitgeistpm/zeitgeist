#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

"$SCRIPT_DIR/cargo_deny.sh"
"$SCRIPT_DIR/cargo_audit.sh"
"$SCRIPT_DIR/semgrep.sh"
"$SCRIPT_DIR/node_checks.sh"
"$SCRIPT_DIR/gitleaks.sh"
"$SCRIPT_DIR/sbom.sh"
"$SCRIPT_DIR/scancode.sh"
