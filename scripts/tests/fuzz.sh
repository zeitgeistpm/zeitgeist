#!/usr/bin/env bash

# Fuzzing tests

set -euxo pipefail

# Using specific_run = (RUN * FUZZ_FACT) / BASE allows us to specify
# a hardware- and fuzz target specific run count.
BASE=1000
RUNS="${RUNS:-50000}"
FEE_SIGMOID_FACT=50000

# --- Prediction Market Pallet fuzz tests ---
RUST_BACKTRACE=1 cargo fuzz run --fuzz-dir zrml/prediction-markets/fuzz pm_full_workflow -- -runs=$RUNS

# --- Swaps Pallet fuzz tests ---
RUST_BACKTRACE=1 cargo fuzz run --fuzz-dir zrml/swaps/fuzz swaps_full_workflow -- -runs=$RUNS

# --- Orderbook-v1 Pallet fuzz tests ---
RUST_BACKTRACE=1 cargo fuzz run --fuzz-dir zrml/orderbook-v1/fuzz orderbook_v1_full_workflow -- -runs=$RUNS

# --- Rikiddo Pallet fuzz tests ---
# Release is required here since it triggers debug assertions otherwise
RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz fee_sigmoid -- -runs=$(($(($RUNS * $FEE_SIGMOID_FACT)) / $BASE))

