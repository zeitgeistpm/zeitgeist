#!/usr/bin/env bash

# Creates a local relay chain with two validators and one parachain
#
# It is still necessary to register Zeitgeist through a parathread or sudoScheduleParaInitialize

POLKADOT_DIR="target/polkadot"

if ! [ -d $POLKADOT_DIR ]; then
  git clone --single-branch --branch rococo-v1 https://github.com/paritytech/polkadot $POLKADOT_DIR
fi

cd $POLKADOT_DIR
git fetch origin
git rebase origin/rococo-v1

# Build everything

cargo build --release
cargo build --features parachain --manifest-path ../../node/Cargo.toml --release

# Set-up

../release/zeitgeist export-genesis-state --chain battery_park --parachain-id 9123 > para-genesis
../release/zeitgeist export-genesis-wasm --chain battery_park > para-wasm

# Polkadot validators

./target/release/polkadot build-spec --chain rococo-local --disable-default-bootnode --raw > rococo_local.json
./target/release/polkadot \
  --alice \
  --chain ./rococo_local.json \
  --port 30000 \
  --rpc-port 8000 \
  --tmp \
  --validator \
  --ws-port 9000 \
  & node_pid=$!
./target/release/polkadot \
  --bob \
  --chain ./rococo_local.json \
  --port 30001 \
  --rpc-port 8001 \
  --tmp \
  --validator \
  --ws-port 9001 \
  & node_pid=$!

# Zeitgeist collators

../release/zeitgeist \
  --alice \
  --chain battery_park \
  --collator \
  --parachain-id 9123 \
  --tmp \
  -- \
  --chain ./rococo_local.json \
  --execution wasm \
  --port 31000 \
  --rpc-port 8100 \
  --tmp \
  --ws-port 9100
