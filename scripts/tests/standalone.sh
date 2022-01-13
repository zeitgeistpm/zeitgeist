#!/usr/bin/env bash

# Standalone node: Runtime and node directories

set -euxo pipefail

. "$(dirname "$0")/aux-functions.sh" --source-only

check_package_with_feature runtime std
check_package_with_feature runtime std,runtime-benchmarks

# check_package_with_feature node default
# check_package_with_feature node default,runtime-benchmarks
