#!/usr/bin/env bash

# Parachain node: Runtime and node directories

set -euxo pipefail

. "$(dirname "$0")/aux-functions.sh" --source-only

check_package_with_feature runtime/battery-station parachain,runtime-benchmarks,std,try-runtime

check_package_with_feature runtime/zeitgeist parachain,runtime-benchmarks,std,try-runtime

check_package_with_feature node default,parachain,runtime-benchmarks,try-runtime
