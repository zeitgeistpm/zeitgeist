#!/usr/bin/env bash

# Tests: Run tests on all crates

set -euxo pipefail

. "$(dirname "$0")/aux-functions.sh" --source-only

# Test standalone
test_package_with_feature "." "default,runtime-benchmarks" ""

# Test parachain
test_package_with_feature "." "default,parachain,runtime-benchmarks" ""