#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later

# Tests: Run tests on all crates using a parachain build

set -euxo pipefail

. "$(dirname "$0")/aux-functions.sh" --source-only

# Test parachain
test_package_with_feature "." "default,parachain,runtime-benchmarks" ""