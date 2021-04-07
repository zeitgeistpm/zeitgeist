#!/usr/bin/env bash

POLKADOT_DIR="target/polkadot"

if ! [ -d $POLKADOT_DIR ]; then
  git clone --single-branch --branch rococo-v1 https://github.com/paritytech/polkadot $POLKADOT_DIR
fi

cd $POLKADOT_DIR
git fetch origin
git rebase origin/rococo-v1

cargo build --features real-overseer
./target/debug/polkadot build-spec --chain rococo-local --raw --disable-default-bootnode > rococo_local.json
./target/debug/polkadot --chain ./rococo_local.json -d cumulus_relay1 --validator --bob --port 50555 & node_pid=$!
./target/debug/polkadot --chain ./rococo_local.json -d cumulus_relay0 --validator --alice --port 50556 & node_pid=$!

cargo run --features parachain --manifest-path ../../node/Cargo.toml -- -d local-test --collator --alice --ws-port 9945 --parachain-id 200 -- --chain rococo_local.json