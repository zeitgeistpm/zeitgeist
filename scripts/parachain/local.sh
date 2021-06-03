#!/usr/bin/env bash

# Creates a local relay chain for testing and development

set -euxo pipefail

CHAIN=local
PARACHAIN_ID=9123
POLKADOT_DIR="target/polkadot"

if ! [ -d $POLKADOT_DIR ]; then
  git clone https://github.com/paritytech/polkadot $POLKADOT_DIR
fi

cd $POLKADOT_DIR
git checkout --track origin/release-v0.9.3 &> /dev/null || true
git fetch origin
git rebase origin/release-v0.9.3

# Build everything

cargo build --release
cargo build --features parachain --manifest-path ../../node/Cargo.toml --release

# Set-up

../release/zeitgeist export-genesis-state --chain $CHAIN --parachain-id $PARACHAIN_ID > para-genesis
../release/zeitgeist export-genesis-wasm --chain $CHAIN > para-wasm

./target/release/polkadot build-spec --chain rococo-local --disable-default-bootnode > rococo-local-plain.json
set +x
echo "s/\"paras\": \[\]/\"paras\": [[$PARACHAIN_ID, { \"genesis_head\": \"$(cat para-genesis)\", \"validation_code\": \"$(cat para-wasm)\", \"parachain\": true }]]/" > sed.txt
set -x
sed -f sed.txt rococo-local-plain.json > rococo-local-plain-parachain.json
./target/release/polkadot build-spec --chain rococo-local-plain-parachain.json --disable-default-bootnode --raw > rococo-local-raw.json

../release/zeitgeist build-spec --chain $CHAIN --disable-default-bootnode > zeitgeist-plain.json
../release/zeitgeist build-spec --chain zeitgeist-plain.json --disable-default-bootnode --raw > zeitgeist-raw.json

# Polkadot validators

start_validator() {
  local author=$1
  local port=$2
  local rpc_port=$3
  local ws_port=$4

  ./target/release/polkadot \
    $author \
    --chain=./rococo-local-raw.json \
    --port=$port \
    --rpc-port=$rpc_port \
    --tmp \
    --ws-port=$ws_port \
    -lruntime=trace
}

# Feel free to comment, add or remove validators. Just remember that #Validators > #Collators 

start_validator --alice 31000 8100 9100 &> /dev/null & node_pid=$!
start_validator --bob 31001 8101 9101 &> /dev/null & node_pid=$!
start_validator --charlie 31002 8102 9102 & node_pid=$!

# Zeitgeist collators

start_collator() {
  local author=$1

  local collator_port=$2
  local collator_rpc_port=$3
  local collator_ws_port=$4

  local relay_chain_port=$5
  local relay_chain_rpc_port=$6
  local relay_chain_ws_port=$7

  ../release/zeitgeist \
    $author \
    --chain=./zeitgeist-raw.json \
    --collator \
    --parachain-id=$PARACHAIN_ID \
    --port=$collator_port \
    --rpc-port $collator_rpc_port \
    --tmp \
    --ws-port=$collator_ws_port \
    -lauthor-filter=trace \
    -lauthor-inherent=trace \
    -lruntime=trace \
    -- \
    --chain=./rococo-local-raw.json \
    --execution=wasm \
    --port=$relay_chain_port \
    --rpc-port=$relay_chain_rpc_port \
    --tmp \
    --ws-port=$relay_chain_ws_port \
    -lcumulus_collator=trace \
    -lruntime=trace
}

# Feel free to comment, add or remove collators. Just remember that #Validators > #Collators

start_collator --alice 30333 9933 9944 32000 8200 9200 &> /dev/null & node_pid=$!
start_collator --bob 30334 9934 9945 32001 8201 9201