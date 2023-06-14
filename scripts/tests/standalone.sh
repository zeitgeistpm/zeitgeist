#!/usr/bin/env bash

# Standalone node: Runtime and node directories

set -euxo pipefail

. "$(dirname "$0")/aux-functions.sh" --source-only

check_package_with_feature runtime/battery-station runtime-benchmarks,std,try-runtime

check_package_with_feature runtime/zeitgeist runtime-benchmarks,std,try-runtime

check_package_with_feature node default,runtime-benchmarks,try-runtime
