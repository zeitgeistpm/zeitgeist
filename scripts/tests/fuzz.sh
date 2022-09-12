#!/usr/bin/env bash

# Fuzzing tests

set -euxo pipefail

# Using specific_run = (RUN * FUZZ_FACT) / BASE allows us to specify
# a hardware- and fuzz target specific run count.
BASE=1000
RUNS="${RUNS:-50000}"

CREATE_POOL_FACT=500
POOL_JOIN_FACT=150
POOL_JOIN_WITH_EXACT_POOL_AMOUNT_FACT=150
POOL_JOIN_WITH_EXACT_ASSET_AMOUNT_FACT=150
SWAP_EXACT_AMOUNT_IN_FACT=500
SWAP_EXACT_AMOUNT_OUT_FACT=500
POOL_EXIT_WITH_EXACT_ASSET_AMOUNT_FACT=150
POOL_EXIT_WITH_EXACT_POOL_AMOUNT_FACT=150
POOL_EXIT_FACT=150

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
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/prediction-markets/fuzz pm_full_workflow -- -runs=$RUNS

# --- Swaps Pallet fuzz tests ---
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/swaps/fuzz create_pool -- -runs=$(($(($RUNS * $CREATE_POOL_FACT)) / $BASE))
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/swaps/fuzz pool_join -- -runs=$(($(($RUNS * $POOL_JOIN_FACT)) / $BASE))
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/swaps/fuzz pool_join_with_exact_pool_amount -- -runs=$(($(($RUNS * $POOL_JOIN_WITH_EXACT_POOL_AMOUNT_FACT)) / $BASE))
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/swaps/fuzz pool_join_with_exact_asset_amount -- -runs=$(($(($RUNS * $POOL_JOIN_WITH_EXACT_ASSET_AMOUNT_FACT)) / $BASE))
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/swaps/fuzz swap_exact_amount_in -- -runs=$(($(($RUNS * $SWAP_EXACT_AMOUNT_IN_FACT)) / $BASE))
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/swaps/fuzz swap_exact_amount_out -- -runs=$(($(($RUNS * $SWAP_EXACT_AMOUNT_OUT_FACT)) / $BASE))
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/swaps/fuzz pool_exit_with_exact_asset_amount -- -runs=$(($(($RUNS * $POOL_EXIT_WITH_EXACT_ASSET_AMOUNT_FACT)) / $BASE))
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/swaps/fuzz pool_exit_with_exact_pool_amount -- -runs=$(($(($RUNS * $POOL_EXIT_WITH_EXACT_POOL_AMOUNT_FACT)) / $BASE))
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/swaps/fuzz pool_exit -- -runs=$(($(($RUNS * $POOL_EXIT_FACT)) / $BASE))

# --- Orderbook-v1 Pallet fuzz tests ---
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/orderbook-v1/fuzz orderbook_v1_full_workflow -- -runs=$RUNS

# --- Rikiddo Pallet fuzz tests ---
# Release is required here since it triggers debug assertions otherwise
# Using the default RUNS multiplier, each fuzz test needs approx. 6-7 seconds
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz fee_sigmoid -- -runs=$(($(($RUNS * $FEE_SIGMOID_FACT)) / $BASE))
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz fixedi_to_fixedu_conversion -- -runs=$(($(($RUNS * $FIXEDI_TO_FIXEDU_FACT)) / $BASE))
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz fixedu_to_fixedi_conversion -- -runs=$(($(($RUNS * $FIXEDU_TO_FIXEDI_FACT)) / $BASE))
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz balance_to_fixedu_conversion -- -runs=$(($(($RUNS * $BALANCE_TO_FIXEDU_FACT)) / $BASE))
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz fixedu_to_balance_conversion -- -runs=$(($(($RUNS * $FIXEDU_TO_BALANCE_FACT)) / $BASE))
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz ema_market_volume_first_state -- -runs=$(($(($RUNS * $EMA_MARKET_VOLUME_FIRST_STATE_FACT)) / $BASE))
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz ema_market_volume_second_state -- -runs=$(($(($RUNS * $EMA_MARKET_VOLUME_SECOND_STATE_FACT)) / $BASE))
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz ema_market_volume_third_state -- -runs=$(($(($RUNS * $EMA_MARKET_VOLUME_THIRD_STATE_FACT)) / $BASE))
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz ema_market_volume_estimate_ema -- -runs=$(($(($RUNS * $EMA_MARKET_VOLUME_ESTIMATE_EMA_FACT)) / $BASE))
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz rikiddo_with_initial_fee -- -runs=$(($(($RUNS * $RIKIDDO_WITH_INITIAL_FEE_FACT)) / $BASE))
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz rikiddo_with_calculated_fee -- -runs=$(($(($RUNS * $RIKIDDO_WITH_CALCULATED_FEE_FACT)) / $BASE))
# This actually needs approx. 107 seconds. Need to find a way to optimize fuzzing on-chain
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-fuzz-%p-%m.profraw' RUST_BACKTRACE=1 cargo fuzz run --release --fuzz-dir zrml/rikiddo/fuzz rikiddo_pallet -- -runs=$(($(($RUNS * $RIKIDDO_PALLET_FACT)) / $BASE))

mkdir tmp
grcov . --binary-path ./target/debug/deps/ -s . -t lcov --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o tmp/fuzz.lcov