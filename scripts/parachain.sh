#!/usr/bin/env bash

POLKADOT_DIR="target/polkadot"

if ! [ -d $POLKADOT_DIR ]; then
  git clone --single-branch --branch rococo-v1 https://github.com/paritytech/polkadot $POLKADOT_DIR
fi

cd $POLKADOT_DIR
git fetch origin
git rebase origin/rococo-v1

# Build everything

cargo build --features real-overseer --release
cargo build --features parachain --manifest-path ../../node/Cargo.toml --release

# Set-up

../release/zeitgeist export-genesis-state --parachain-id 200 > para-200-genesis
../release/zeitgeist export-genesis-wasm > para-200-wasm

# Polkadot validators

./target/release/polkadot build-spec --chain rococo-local --disable-default-bootnode --raw > rococo_local.json
./target/release/polkadot \
  --alice \
  --chain ./rococo_local.json \
  -d cumulus_relay0 \
  --port 30000 \
  --validator \
  --ws-port 9000 \
  & node_pid=$!
./target/release/polkadot \
  --bob \
  --chain ./rococo_local.json \
  -d cumulus_relay1 \
  --port 30001 \
  --validator \
  --ws-port 9001 \
  & node_pid=$!

# Zeitgeist collators

../release/zeitgeist \
  --alice \
  --collator \
  -d local-test \
  --parachain-id 200 \
  -- \
  --chain ./rococo_local.json \
  --execution wasm \
  --port 31000 \
  --ws-port 9100
