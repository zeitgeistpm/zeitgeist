#!/usr/bin/env bash

# Fuzzing tests

set -euxo pipefail

RUNS="${RUNS:-50000}"

RUST_BACKTRACE=1 cargo fuzz run --fuzz-dir zrml/prediction-markets/fuzz full_workflow -- -runs=$RUNS

RUST_BACKTRACE=1 cargo fuzz run --fuzz-dir zrml/swaps/fuzz full_workflow -- -runs=$RUNS
