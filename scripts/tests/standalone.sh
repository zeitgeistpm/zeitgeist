#!/usr/bin/env bash

# Standalone node: Runtime and node directories

set -euxo pipefail

. "$(dirname "$0")/aux-functions.sh" --source-only

check_package_with_feature runtime/battery-station std,runtime-benchmarks,try-runtime

check_package_with_feature runtime/zeitgeist std,runtime-benchmarks,try-runtime

check_package_with_feature node default,runtime-benchmarks,try-runtime
