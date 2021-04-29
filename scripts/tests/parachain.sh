#!/usr/bin/env bash

# Parachain node: Runtime and node directories

set -euxo pipefail

. "$(dirname "$0")/aux-functions.sh" --source-only

check_package_with_feature runtime std,parachain

check_package_with_feature node default,parachain
