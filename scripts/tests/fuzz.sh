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
EMA_MARKET_VOLUME_FIRST_STATE_FACT=5000
EMA_MARKET_VOLUME_SECOND_STATE_FACT=7000
EMA_MARKET_VOLUME_THIRD_STATE_FACT=7000
EMA_MARKET_VOLUME_ESTIMATE_EMA_FACT=7000
RIKIDDO_WITH_INITIAL_FEE_FACT=2300
RIKIDDO_WITH_CALCULATED_FEE_FACT=1750
RIKIDDO_PALLET_FACT=1000

# --- Prediction Market Pallet fuzz tests ---
RUST_BACKTRACE=1 cargo fuzz run --fuzz-dir zrml/prediction-markets/fuzz pm_full_workflow -- -runs=$RUNS

# --- Swaps Pallet fuzz tests ---
RUST_BACKTRACE=1 cargo fuzz run --fuzz-dir zrml/swaps/fuzz pool_creation -- -runs=$RUNS
RUST_BACKTRACE=1 cargo fuzz run --fuzz-dir zrml/swaps/fuzz general_pool_joining -- -runs=$RUNS
RUST_BACKTRACE=1 cargo fuzz run --fuzz-dir zrml/swaps/fuzz exact_amount_pool_joining -- -runs=$RUNS
RUST_BACKTRACE=1 cargo fuzz run --fuzz-dir zrml/swaps/fuzz exact_asset_amount_pool_joining -- -runs=$RUNS
RUST_BACKTRACE=1 cargo fuzz run --fuzz-dir zrml/swaps/fuzz input_swap_exact_amount_pool_joining -- -runs=$RUNS
RUST_BACKTRACE=1 cargo fuzz run --fuzz-dir zrml/swaps/fuzz output_swap_exact_amount_pool_joining -- -runs=$RUNS
RUST_BACKTRACE=1 cargo fuzz run --fuzz-dir zrml/swaps/fuzz exact_asset_amount_pool_exiting -- -runs=$RUNS
RUST_BACKTRACE=1 cargo fuzz run --fuzz-dir zrml/swaps/fuzz exact_amount_pool_exiting -- -runs=$RUNS
RUST_BACKTRACE=1 cargo fuzz run --fuzz-dir zrml/swaps/fuzz general_pool_exiting -- -runs=$RUNS

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
RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz ema_market_volume_first_state -- -runs=$(($(($RUNS * $EMA_MARKET_VOLUME_FIRST_STATE_FACT)) / $BASE))
RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz ema_market_volume_second_state -- -runs=$(($(($RUNS * $EMA_MARKET_VOLUME_SECOND_STATE_FACT)) / $BASE))
RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz ema_market_volume_third_state -- -runs=$(($(($RUNS * $EMA_MARKET_VOLUME_THIRD_STATE_FACT)) / $BASE))
RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz ema_market_volume_estimate_ema -- -runs=$(($(($RUNS * $EMA_MARKET_VOLUME_ESTIMATE_EMA_FACT)) / $BASE))
RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz rikiddo_with_initial_fee -- -runs=$(($(($RUNS * $RIKIDDO_WITH_INITIAL_FEE_FACT)) / $BASE))
RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz rikiddo_with_calculated_fee -- -runs=$(($(($RUNS * $RIKIDDO_WITH_CALCULATED_FEE_FACT)) / $BASE))
# This actually needs approx. 107 seconds. Need to find a way to optimize fuzzing on-chain
RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz rikiddo_pallet -- -runs=$(($(($RUNS * $RIKIDDO_PALLET_FACT)) / $BASE))