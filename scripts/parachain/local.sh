#!/usr/bin/env bash

# Creates a local relay chain with two validators and one parachain

set -euxo pipefail

CHAIN=local
PARACHAIN_ID=9123
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

../release/zeitgeist export-genesis-state --chain $CHAIN --parachain-id $PARACHAIN_ID > para-genesis
../release/zeitgeist export-genesis-wasm --chain $CHAIN > para-wasm

./target/release/polkadot build-spec --chain rococo-local --disable-default-bootnode > rococo-local-plain.json
echo "s/\"paras\": \[\]/\"paras\": [[9123, { \"genesis_head\": \"$(cat para-genesis)\", \"validation_code\": \"$(cat para-wasm)\", \"parachain\": true }]]/" > sed.txt
sed -f sed.txt rococo-local-plain.json > rococo-local-plain-parachain.json
./target/release/polkadot build-spec --chain rococo-local-plain-parachain.json --disable-default-bootnode --raw > rococo-local-raw.json

../release/zeitgeist build-spec --chain $CHAIN --disable-default-bootnode > zeitgeist-plain.json
../release/zeitgeist build-spec --chain zeitgeist-plain.json --disable-default-bootnode --raw > zeitgeist-raw.json

# Polkadot validators

start_validator() {
  local actor=$1
  local port=$2
  local rpc_port=$3
  local ws_port=$4

  ./target/release/polkadot \
    --$actor \
    --chain=./rococo-local-raw.json \
    --port=$port \
    --rpc-port $rpc_port \
    --tmp \
    --ws-port=$ws_port \
    -lruntime=trace \
    & node_pid=$!
}

start_validator alice 30000 8000 9000
start_validator bob 30001 8001 9001

# Zeitgeist collators

../release/zeitgeist \
  --chain=./zeitgeist-raw.json \
  --collator \
  --parachain-id=$PARACHAIN_ID \
  --port=31000 \
  --rpc-port 8100 \
  --tmp \
  --ws-port=9100 \
  -lruntime=trace \
  -- \
  --chain=./rococo-local-raw.json \
  --execution=wasm
