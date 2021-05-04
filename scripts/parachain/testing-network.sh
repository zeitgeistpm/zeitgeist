#!/usr/bin/env bash

# Not meant to be called directly.
#
# Common script used by live testing network (Rococo, Chachacha, etc) scipts. Spin-ups 2
# validators and one parachain using Docker.
#
# Only required variable is "RELAY_CHAIN_SPEC_FILE", everything else defaults to common or
# Chachacha values if not specified.

set -euxo pipefail

# Variables

CHACHACHA_BOOTNODES="--bootnodes /ip4/34.89.248.129/tcp/30333/p2p/12D3KooWD8CAZBgpeZiSVVbaj8mijR6mfgUsHNAmCKwsRoRnFod4 --bootnodes /ip4/35.242.217.240/tcp/30333/p2p/12D3KooWBthdCz4JshkMb4GxJXVwrHPv9GpWAgfh2hAdkyXQDKyN"

BOOTNODES="${BOOTNODES:-$CHACHACHA_BOOTNODES}"
DOCKER_POLKADOT_BIN="${DOCKER_POLKADOT_BIN:-/usr/local/bin/polkadot}"
DATA_DIR="${DATA_DIR:-$HOME/rococo}"
PARACHAIN_CHAIN="${PARACHAIN_CHAIN:-battery_park}"
PARACHAIN_ID="${PARACHAIN_ID:-9123}"
VALIDATOR_CHAIN="${VALIDATOR_CHAIN:-rococo-chachacha}"

PARACHAIN_IMAGE="${PARACHAIN_IMAGE:-zeitgeistpm/zeitgeist-node-parachain}"
POLKADOT_IMAGE="${POLKADOT_IMAGE:-centrifugeio/rococo:chachacha-v1}"

PARACHAIN_0="${PARACHAIN_0:-zeitgeist-chachacha-parachain-0}"
PARACHAIN_0_PORT="${PARACHAIN_0_PORT:-30000}"
PARACHAIN_0_RPC_PORT="${PARACHAIN_0_RPC_PORT:-8000}"
PARACHAIN_0_WS_PORT="${PARACHAIN_0_WS_PORT:-9000}"

VALIDATOR_0="${VALIDATOR_0:-zeitgeist-chachacha-validator-0}"
VALIDATOR_0_PORT="${VALIDATOR_0_PORT:-31000}"
VALIDATOR_0_RPC_PORT="${VALIDATOR_0_RPC_PORT:-8100}"
VALIDATOR_0_WS_PORT="${VALIDATOR_0_WS_PORT:-9100}"

VALIDATOR_1="${VALIDATOR_1:-zeitgeist-chachacha-validator-1}"
VALIDATOR_1_PORT="${VALIDATOR_1_PORT:-31001}"
VALIDATOR_1_RPC_PORT="${VALIDATOR_1_RPC_PORT:-8101}"
VALIDATOR_1_WS_PORT="${VALIDATOR_1_WS_PORT:-9101}"

sudo apt update
sudo apt install -y curl docker.io
sudo docker stop $PARACHAIN_0 $VALIDATOR_0 $VALIDATOR_1 &> /dev/null || true
sudo docker rm $PARACHAIN_0 $VALIDATOR_0 $VALIDATOR_1 &> /dev/null || true
sudo docker pull $PARACHAIN_IMAGE
sudo docker pull $POLKADOT_IMAGE
mkdir -p $DATA_DIR/$VALIDATOR_0 $DATA_DIR/$VALIDATOR_1 $DATA_DIR/$PARACHAIN_0

# Functions

delete_validator() {
    local container_name=$1

    sudo docker stop $container_name
    sudo docker rm $container_name
}

generate_rotate_key() {
    local rpc_port=$1

    sleep 2
    ROTATE_KEY=$(curl -H 'Content-Type:application/json;charset=utf-8' --data '{ "id":1, "jsonrpc":"2.0", "method":"author_rotateKeys" }' localhost:$rpc_port)
    echo $ROTATE_KEY
}

launch_validator() {
    local container_name=$1
    local docker_extra_params=$2
    local validator_extra_params=$3

    sudo docker run \
        -d \
        $docker_extra_params \
        -v $DATA_DIR/$container_name:/data \
        --name $container_name \
        --restart always \
        $POLKADOT_IMAGE \
        $DOCKER_POLKADOT_BIN \
        $BOOTNODES \
        --chain $VALIDATOR_CHAIN \
        --name $container_name \
        --validator \
        $validator_extra_params
}

# Validators

launch_validator "$VALIDATOR_0" "-p $VALIDATOR_0_PORT:30333 -p $VALIDATOR_0_RPC_PORT:9933 -p $VALIDATOR_0_WS_PORT:9944" "--rpc-cors all --rpc-methods=Unsafe --unsafe-rpc-external"
ROTATE_KEY_0=$(generate_rotate_key $VALIDATOR_0_RPC_PORT)
delete_validator "$VALIDATOR_0"
launch_validator "$VALIDATOR_0" "-p $VALIDATOR_0_PORT:30333 -p $VALIDATOR_0_RPC_PORT:9933 -p $VALIDATOR_0_WS_PORT:9944" ""

launch_validator "$VALIDATOR_1" "-p $VALIDATOR_1_PORT:30333 -p $VALIDATOR_1_RPC_PORT:9933 -p $VALIDATOR_1_WS_PORT:9944" "--rpc-cors all --rpc-methods=Unsafe --unsafe-rpc-external"
ROTATE_KEY_1=$(generate_rotate_key $VALIDATOR_1_RPC_PORT)
delete_validator "$VALIDATOR_1"
launch_validator "$VALIDATOR_1" "-p $VALIDATOR_1_PORT:30333 -p $VALIDATOR_1_RPC_PORT:9933 -p $VALIDATOR_1_WS_PORT:9944" ""

# Parachains

sudo docker run \
    --rm \
    $PARACHAIN_IMAGE \
    export-genesis-state \
    --chain $PARACHAIN_CHAIN \
    --parachain-id $PARACHAIN_ID > $DATA_DIR/zeitgeist-genesis-state
sudo docker run \
    --rm \
    $PARACHAIN_IMAGE \
    export-genesis-wasm \
    --chain $PARACHAIN_CHAIN > $DATA_DIR/zeitgeist-genesis-wasm
cp $RELAY_CHAIN_SPEC_FILE $DATA_DIR/$PARACHAIN_0/relay-chain-spec.json

sudo docker run \
    -d \
    -p $PARACHAIN_0_PORT:30333 \
    -p $PARACHAIN_0_RPC_PORT:9933 \
    -p $PARACHAIN_0_WS_PORT:9944 \
    -v $DATA_DIR/$PARACHAIN_0:/zeitgeist/data \
    --name $PARACHAIN_0 \
    --restart always \
    $PARACHAIN_IMAGE \
    --chain $PARACHAIN_CHAIN \
    --collator \
    --parachain-id $PARACHAIN_ID \
    --rpc-cors all \
    --rpc-external \
    --ws-external \
    -- \
    --chain /zeitgeist/data/relay-chain-spec.json \
    --execution wasm

echo $ROTATE_KEY_0
echo $ROTATE_KEY_1
