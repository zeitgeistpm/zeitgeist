#!/usr/bin/env bash

# Parachain node: Runtime and node directories

set -euxo pipefail

. "$(dirname "$0")/aux-functions.sh" --source-only

check_package_with_feature runtime/battery-station 'parachain --all-features'

check_package_with_feature runtime/zeitgeist 'parachain --all-features'

check_package_with_feature node 'parachain --all-features'
