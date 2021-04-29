#!/usr/bin/env bash

# Standalone node: Runtime and node directories

set -euxo pipefail

. "$(dirname "$0")/aux-functions.sh" --source-only

build_package_with_feature runtime std

build_package_with_feature node default
