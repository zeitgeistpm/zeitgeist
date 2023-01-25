#!/usr/bin/env bash

# This script benchmarks every pallet that is used within Zeitgeist.
# Execute from the root of the project.

set -eou pipefail

# Configuration

if [ ! -d "./scripts/benchmarks" ]; then
    echo "Please execute this script from the root of the Zeitgeist project folder"
    exit 1
fi;

source ./scripts/benchmarks/configuration.sh

# Standalone benchmarks

cargo build --profile=$PROFILE --features=runtime-benchmarks,$ADDITIONAL_FEATURES --bin=zeitgeist

for pallet in ${FRAME_PALLETS[@]}; do
    ./target/$PROFILE_DIR/zeitgeist benchmark pallet --chain=dev --steps=$FRAME_PALLETS_STEPS --repeat=$FRAME_PALLETS_RUNS --pallet=$pallet --extrinsic='*' --execution=$EXECUTION $ADDITIONAL --wasm-execution=compiled --heap-pages=4096 --template=$FRAME_WEIGHT_TEMPLATE --output=$EXTERNAL_WEIGHTS_PATH
done

for pallet in ${ORML_PALLETS[@]}; do
    ./target/$PROFILE_DIR/zeitgeist benchmark pallet --chain=dev --steps=$ORML_PALLETS_STEPS --repeat=$ORML_PALLETS_RUNS --pallet=$pallet --extrinsic='*' --execution=$EXECUTION $ADDITIONAL --wasm-execution=compiled --heap-pages=4096 --template=$ORML_WEIGHT_TEMPLATE --output=$EXTERNAL_WEIGHTS_PATH
done

for pallet in ${ZEITGEIST_PALLETS[@]}; do
    pallet_folder_name=${pallet//zrml_/}
    pallet_folder_name=${pallet_folder_name//_/-}
    ./target/$PROFILE_DIR/zeitgeist benchmark pallet --chain=dev --steps=$ZEITGEIST_PALLETS_STEPS --repeat=$ZEITGEIST_PALLETS_RUNS --pallet=$pallet --extrinsic='*' --execution=$EXECUTION $ADDITIONAL --wasm-execution=compiled --heap-pages=4096 --template=$ZEITGEIST_WEIGHT_TEMPLATE --output=./zrml/$pallet_folder_name/src/weights.rs
done

# Parachain benchmarks

cargo build --profile=$PROFILE --features=runtime-benchmarks,parachain,$ADDITIONAL_FEATURES --bin=zeitgeist

for pallet in ${FRAME_PALLETS_PARACHAIN[@]}; do
    ./target/$PROFILE_DIR/zeitgeist benchmark pallet --chain=dev --steps=$FRAME_PALLETS_PARACHAIN_STEPS --repeat=$FRAME_PALLETS_PARACHAIN_RUNS --pallet=$pallet --extrinsic='*' --execution=$EXECUTION $ADDITIONAL --wasm-execution=compiled --heap-pages=4096 --template=$FRAME_WEIGHT_TEMPLATE --output=$EXTERNAL_WEIGHTS_PATH
done

cargo fmt
