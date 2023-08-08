#!/usr/bin/env bash

# Tests: Run tests on all crates using a standalone build

set -euxo pipefail

. "$(dirname "$0")/aux-functions.sh" --source-only

# Test standalone
test_package_with_feature "." "default,runtime-benchmarks" ""