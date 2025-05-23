#!/usr/bin/env bash

# Fuzzing tests

verbose=""

for arg in "$@"; do
    case $arg in
    "--verbose" | "-v")
        verbose="verbose"
        ;;
    *)
        echo "Unknown option '$arg'"
        usage
        exit 1
        ;;
    esac
done

set -euxo pipefail

if [[ ${verbose} = "verbose" ]]; then
    echo "Its verbose"
    RUST_BACKTRACE=1
fi

# Fuzzing tests

# Using specific_run = (RUN * FUZZ_FACT) / BASE allows us to specify
# a hardware- and fuzz target specific run count.
BASE=1000
RUNS=1

CREATE_POOL_FACT=500
POOL_JOIN_FACT=150
POOL_JOIN_WITH_EXACT_POOL_AMOUNT_FACT=150
POOL_JOIN_WITH_EXACT_ASSET_AMOUNT_FACT=150
SWAP_EXACT_AMOUNT_IN_FACT=500
SWAP_EXACT_AMOUNT_OUT_FACT=500
POOL_EXIT_WITH_EXACT_ASSET_AMOUNT_FACT=150
POOL_EXIT_WITH_EXACT_POOL_AMOUNT_FACT=150
POOL_EXIT_FACT=150

# --- Prediction Market Pallet fuzz tests ---
cargo fuzz run --release --fuzz-dir zrml/prediction-markets/fuzz pm_full_workflow -- -runs=$RUNS

# --- Swaps Pallet fuzz tests ---
cargo fuzz run --release --fuzz-dir zrml/swaps/fuzz create_pool -- -runs=$(($(($RUNS * $CREATE_POOL_FACT)) / $BASE))
cargo fuzz run --release --fuzz-dir zrml/swaps/fuzz pool_join -- -runs=$(($(($RUNS * $POOL_JOIN_FACT)) / $BASE))
cargo fuzz run --release --fuzz-dir zrml/swaps/fuzz pool_join_with_exact_pool_amount -- -runs=$(($(($RUNS * $POOL_JOIN_WITH_EXACT_POOL_AMOUNT_FACT)) / $BASE))
cargo fuzz run --release --fuzz-dir zrml/swaps/fuzz pool_join_with_exact_asset_amount -- -runs=$(($(($RUNS * $POOL_JOIN_WITH_EXACT_ASSET_AMOUNT_FACT)) / $BASE))
cargo fuzz run --release --fuzz-dir zrml/swaps/fuzz swap_exact_amount_in -- -runs=$(($(($RUNS * $SWAP_EXACT_AMOUNT_IN_FACT)) / $BASE))
cargo fuzz run --release --fuzz-dir zrml/swaps/fuzz swap_exact_amount_out -- -runs=$(($(($RUNS * $SWAP_EXACT_AMOUNT_OUT_FACT)) / $BASE))
cargo fuzz run --release --fuzz-dir zrml/swaps/fuzz pool_exit_with_exact_asset_amount -- -runs=$(($(($RUNS * $POOL_EXIT_WITH_EXACT_ASSET_AMOUNT_FACT)) / $BASE))
cargo fuzz run --release --fuzz-dir zrml/swaps/fuzz pool_exit_with_exact_pool_amount -- -runs=$(($(($RUNS * $POOL_EXIT_WITH_EXACT_POOL_AMOUNT_FACT)) / $BASE))
cargo fuzz run --release --fuzz-dir zrml/swaps/fuzz pool_exit -- -runs=$(($(($RUNS * $POOL_EXIT_FACT)) / $BASE))

# --- Orderbook-v1 Pallet fuzz tests ---
cargo fuzz run --release --fuzz-dir zrml/orderbook/fuzz orderbook_v1_full_workflow -- -runs=$RUNS

cargo fuzz run --release --fuzz-dir zrml/futarchy/fuzz submit_proposal -- -runs=$RUNS

cargo fuzz run --release --fuzz-dir zrml/combinatorial-tokens/fuzz split_position -- -runs=$RUNS
cargo fuzz run --release --fuzz-dir zrml/combinatorial-tokens/fuzz merge_position -- -runs=$RUNS
cargo fuzz run --release --fuzz-dir zrml/combinatorial-tokens/fuzz redeem_position -- -runs=$RUNS

cargo fuzz run --release --fuzz-dir zrml/neo-swaps/fuzz deploy_combinatorial_pool -- -runs=$RUNS
cargo fuzz run --release --fuzz-dir zrml/neo-swaps/fuzz combo_buy -- -runs=$RUNS
cargo fuzz run --release --fuzz-dir zrml/neo-swaps/fuzz combo_sell -- -runs=$RUNS
