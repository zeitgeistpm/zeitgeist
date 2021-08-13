#!/usr/bin/env bash

# Fuzzing tests

set -euxo pipefail

RUNS="${RUNS:-50000}"

RUST_BACKTRACE=1 cargo fuzz run --fuzz-dir zrml/prediction-markets/fuzz pm_full_workflow -- -runs=$RUNS

RUST_BACKTRACE=1 cargo fuzz run --fuzz-dir zrml/swaps/fuzz swaps_full_workflow -- -runs=$RUNS

RUST_BACKTRACE=1 cargo fuzz run --fuzz-dir zrml/orderbook-v1/fuzz orderbook_v1_full_workflow -- -runs=$RUNS
RUST_BACKTRACE=1 cargo fuzz run --fuzz-dir zrml/rikiddo/fuzz rikiddo_full_workflow -- -runs=$RUNS
