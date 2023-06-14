#!/usr/bin/env bash

# Creates a local relay chain for testing and development
#
# IMPORTANT: After initialization, the parachain starts producing blocks in ~1 minute

set -euxo pipefail

PARACHAIN_CHAIN=dev
PARACHAIN_ID=2101
POLKADOT_BRANCH=release-v0.9.11
POLKADOT_DIR="target/polkadot"
RELAYCHAIN_CHAIN=rococo-local

if ! [ -d $POLKADOT_DIR ]; then
  git clone https://github.com/paritytech/polkadot $POLKADOT_DIR
fi

cd $POLKADOT_DIR
git checkout --track origin/$POLKADOT_BRANCH &> /dev/null || true
git fetch origin
git rebase origin/$POLKADOT_BRANCH

# Build everything

cargo build --release
cargo build --bin zeitgeist --features parachain --manifest-path ../../Cargo.toml --release

# Set-up

../release/zeitgeist build-spec --chain $PARACHAIN_CHAIN --disable-default-bootnode > zeitgeist-plain.json
../release/zeitgeist build-spec --chain zeitgeist-plain.json --disable-default-bootnode --raw > zeitgeist-raw.json

../release/zeitgeist export-genesis-state --chain zeitgeist-raw.json --parachain-id $PARACHAIN_ID > para-genesis
../release/zeitgeist export-genesis-wasm --chain zeitgeist-raw.json > para-wasm

./target/release/polkadot build-spec --chain $RELAYCHAIN_CHAIN --disable-default-bootnode > relaychain-plain.json
set +x
echo "s/\"paras\": \[\]/\"paras\": [[$PARACHAIN_ID, { \"genesis_head\": \"$(cat para-genesis)\", \"validation_code\": \"$(cat para-wasm)\", \"parachain\": true }]]/" > sed.txt
set -x
sed -f sed.txt relaychain-plain.json > relaychain-plain-with-parachain.json
./target/release/polkadot build-spec --chain relaychain-plain-with-parachain.json --disable-default-bootnode --raw > relaychain-raw.json

# Polkadot validators

start_validator() {
  local author=$1
  local port=$2
  local rpc_port=$3
  local ws_port=$4

  ./target/release/polkadot \
    $author \
    --chain=./relaychain-raw.json \
    --port=$port \
    --rpc-port=$rpc_port \
    --tmp \
    --ws-port=$ws_port \
    -lruntime=trace
}

# Feel free to comment, add or remove validators. Just remember that #Validators > #Collators 

start_validator --alice 31000 8100 9100 &> /dev/null & node_pid=$!
start_validator --bob 31001 8101 9101 &> /dev/null & node_pid=$!

# Zeitgeist collators

start_collator() {
  local collator_port=$1
  local collator_rpc_port=$2
  local collator_ws_port=$3

  local relay_chain_port=$4
  local relay_chain_rpc_port=$5
  local relay_chain_ws_port=$6

  local seed=$7
  local public_key=$8

  rm -rf /tmp/zeitgeist-parachain-$collator_rpc_port
  rm -rf /tmp/zeitgeist-relaychain-$relay_chain_rpc_port

  LAUNCH_PARACHAIN_CMD="../release/zeitgeist \
    --base-path=/tmp/zeitgeist-parachain-$collator_rpc_port \
    --chain=./zeitgeist-raw.json \
    --collator \
    --parachain-id=$PARACHAIN_ID \
    --port=$collator_port \
    --rpc-port $collator_rpc_port \
    --ws-port=$collator_ws_port \
    -linfo,author_filter=trace,author_inherent=trace,cumulus_collator=trace,executive=trace,filtering_consensus=trace,runtime=trace,staking=trace,txpool=trace \
    -- \
    --base-path=/tmp/zeitgeist-relaychain-$relay_chain_rpc_port \
    --chain=./relaychain-raw.json \
    --execution=wasm \
    --port=$relay_chain_port \
    --rpc-port=$relay_chain_rpc_port \
    --ws-port=$relay_chain_ws_port"

    $LAUNCH_PARACHAIN_CMD & node_pid=$!

    sleep 10
    DATA='{ "id":1, "jsonrpc":"2.0", "method":"author_insertKey", "params":["nmbs", "'"$seed"'", "'"$public_key"'"] }'
    curl -H 'Content-Type: application/json' --data "$DATA" localhost:$collator_rpc_port

    kill $node_pid
    $LAUNCH_PARACHAIN_CMD
}

# Feel free to comment, add or remove collators. Just remember that #Validators > #Collators

start_collator 30333 9933 9944 32000 8200 9200 "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice" "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
