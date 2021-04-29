#!/usr/bin/env bash

# WASM runtimes

set -euxo pipefail

. "$(dirname "$0")/aux-functions.sh" --source-only

WASM_BUILD_TYPE=release cargo build --manifest-path runtime/Cargo.toml
WASM_BUILD_TYPE=release cargo build --features parachain --manifest-path runtime/Cargo.toml