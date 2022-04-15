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

cargo build --release --features=runtime-benchmarks --bin=zeitgeist


for pallet in ${FRAME_PALLETS[@]}; do
    ./target/release/zeitgeist benchmark --chain=dev --steps=$FRAME_PALLETS_STEPS --repeat=$FRAME_PALLETS_RUNS --pallet=$pallet --extrinsic='*' --execution=wasm --wasm-execution=compiled --heap-pages=4096 --template=./misc/frame_weight_template.hbs --output=./runtime/src/weights/
done

cargo fmt

for pallet in ${ORML_PALLETS[@]}; do
    ./target/release/zeitgeist benchmark --chain=dev --steps=$ORML_PALLETS_STEPS --repeat=$ORML_PALLETS_RUNS --pallet=$pallet --extrinsic='*' --execution=wasm --wasm-execution=compiled --heap-pages=4096 --template=./misc/orml_weight_template.hbs --output=./runtime/src/weights/
done

cargo fmt

for pallet in ${ZEITGEIST_PALLETS[@]}; do
    pallet_folder_name=${pallet//zrml_/}
    pallet_folder_name=${pallet_folder_name//_/-}
    ./target/release/zeitgeist benchmark --chain=dev --steps=$ZEITGEIST_PALLETS_STEPS --repeat=$ZEITGEIST_PALLETS_RUNS --pallet=$pallet --extrinsic=* --execution=wasm --wasm-execution=compiled --heap-pages=4096 --template=./misc/weight_template.hbs --output=./zrml/$pallet_folder_name/src/weights.rs
done

cargo fmt

# Parachain benchmarks

cargo build --release --features=parachain,runtime-benchmarks --bin=zeitgeist

for pallet in ${FRAME_PALLETS_PARACHAIN[@]}; do
    ./target/release/zeitgeist benchmark --chain=dev --steps=$FRAME_PALLETS_PARACHAIN_STEPS --repeat=$FRAME_PALLETS_PARACHAIN_RUNS --pallet=$pallet --extrinsic='*' --execution=wasm --wasm-execution=compiled --heap-pages=4096 --template=./misc/frame_weight_template.hbs --output=./runtime/src/weights/
done

cargo fmt