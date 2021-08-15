#!/usr/bin/env bash

# Fuzzing tests

set -euxo pipefail

# Using specific_run = (RUN * FUZZ_FACT) / BASE allows us to specify
# a hardware- and fuzz target specific run count.
BASE=1000
RUNS="${RUNS:-50000}"
FEE_SIGMOID_FACT=50000
FIXEDI_TO_FIXEDU_FACT=100000
FIXEDU_TO_FIXEDI_FACT=100000
BALANCE_TO_FIXEDU_FACT=8000
FIXEDU_TO_BALANCE_FACT=4000

# --- Prediction Market Pallet fuzz tests ---
# RUST_BACKTRACE=1 cargo fuzz run --fuzz-dir zrml/prediction-markets/fuzz pm_full_workflow -- -runs=$RUNS

# --- Swaps Pallet fuzz tests ---
# RUST_BACKTRACE=1 cargo fuzz run --fuzz-dir zrml/swaps/fuzz swaps_full_workflow -- -runs=$RUNS

# --- Orderbook-v1 Pallet fuzz tests ---
RUST_BACKTRACE=1 cargo fuzz run --fuzz-dir zrml/orderbook-v1/fuzz orderbook_v1_full_workflow -- -runs=$RUNS

# --- Rikiddo Pallet fuzz tests ---
# Release is required here since it triggers debug assertions otherwise
# Using the default RUNS multiplier, each fuzz test needs approx. 6-7 seconds
RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz fee_sigmoid -- -runs=$(($(($RUNS * $FEE_SIGMOID_FACT)) / $BASE))
RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz fixedi_to_fixedu_conversion -- -runs=$(($(($RUNS * $FIXEDI_TO_FIXEDU_FACT)) / $BASE))
RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz fixedu_to_fixedi_conversion -- -runs=$(($(($RUNS * $FIXEDU_TO_FIXEDI_FACT)) / $BASE))
RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz balance_to_fixedu_conversion -- -runs=$(($(($RUNS * $BALANCE_TO_FIXEDU_FACT)) / $BASE))
RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz fixedu_to_balance_conversion -- -runs=$(($(($RUNS * $FIXEDU_TO_BALANCE_FACT)) / $BASE))