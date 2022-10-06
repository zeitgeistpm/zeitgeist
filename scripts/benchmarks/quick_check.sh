#!/usr/bin/env bash

# This script benchmarks every pallet that is used within Zeitgeist.
# Execute from the root of the project.

set -eou pipefail

# Configuration

if [ ! -d "./scripts/benchmarks" ]; then
    echo "Please execute this script from the root of the Zeitgeist project folder"
    exit 1
fi;

cargo build --release --features=runtime-benchmarks

./target/release/zeitgeist benchmark pallet --chain dev --execution=native --wasm-execution=compiled --heap-pages=4096 --pallet "*" --extrinsic "*" --steps 1 --repeat 0 --detailed-log-output
