#!/usr/bin/env bash

# This script benchmarks every pallet that is used within Zeitgeist.
# Execute from the root of the project.

# Configuration

FRAME_PALLETS=( frame_system pallet_balances pallet_collective pallet_grandpa pallet_identity \
                pallet_membership pallet_timestamp pallet_treasury pallet_utility pallet_vesting )
FRAME_PALLETS_PARACHAIN=( pallet_author_mapping parachain_staking pallet_crowdloan_rewards )
ORML_PALLETS=( orml_currencies orml_tokens )
ZEITGEIST_PALLETS=( zrml_authorized zrml_court zrml_liquidity_mining zrml_prediction_markets zrml_swaps )


# Standalone benchmarks

cargo build --release --features=runtime-benchmarks


for pallet in ${FRAME_PALLETS[@]}; do
    ./target/release/zeitgeist benchmark --chain=dev --steps=50 --repeat=20 --pallet=$pallet --extrinsic='*' --execution=wasm --wasm-execution=compiled --heap-pages=4096 --template=./misc/frame_weight_template.hbs --output=./runtime/src/weights/
done

for pallet in ${ORML_PALLETS[@]}; do
    ./target/release/zeitgeist benchmark --chain=dev --steps=50 --repeat=20 --pallet=$pallet --extrinsic='*' --execution=wasm --wasm-execution=compiled --heap-pages=4096 --template=./misc/orml_weight_template.hbs --output=./runtime/src/weights/
done

for pallet in ${ZEITGEIST_PALLETS[@]}; do
    pallet_folder_name=${pallet//zrml_//}
    pallet_folder_name=${pallet_folder_name//_/-}
    ./target/release/zeitgeist benchmark --chain=dev --steps=2 --repeat=2 --pallet=$pallet --extrinsic='*' --execution=wasm --wasm-execution=compiled --heap-pages=4096 --template=./misc/weight_template.hbs --output=./zrml/$pallet_folder_name/src/weights.rs
done


# Parachain benchmarks

cargo build --release --features=parachain,runtime-benchmarks

for pallet in ${FRAME_PALLETS_PARACHAIN[@]}; do
    ./target/release/zeitgeist benchmark --chain=dev --steps=50 --repeat=20 --pallet=$pallet --extrinsic='*' --execution=wasm --wasm-execution=compiled --heap-pages=4096 --template=./misc/frame_weight_template.hbs --output=./runtime/src/weights/
done