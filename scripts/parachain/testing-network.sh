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

PARACHAIN_ID="${PARACHAIN_ID:-9123}"
PARACHAIN_IMAGE="${PARACHAIN_IMAGE:-zeitgeistpm/zeitgeist-node-parachain}"
PARACHAINS_NUM=$(($PARACHAINS_NUM - 1))
VALIDATORS_NUM=$(($VALIDATORS_NUM - 1))

# Functions

delete_validator() {
    local container_name=$1

    sudo docker stop $container_name
    sudo docker rm $container_name
}

initial_container_configurations() {
    local container_name=$1

    mkdir -p $DATA_DIR/$container_name
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

# Init

sudo apt update
sudo apt install -y curl docker.io
sudo docker pull $PARACHAIN_IMAGE
sudo docker pull $POLKADOT_IMAGE
sudo docker kill $(sudo docker ps -q)
sudo docker rm $(sudo docker ps -a -q)

# Validators

for idx in $(seq 0 $VALIDATORS_NUM)
do
    LOCAL_CONTAINER_NAME="$VALIDATOR-$idx"
    LOCAL_PORT=$(($VALIDATOR_PORT + $idx))
    LOCAL_RPC_PORT=$(($VALIDATOR_RPC_PORT + $idx))
    LOCAL_WS_PORT=$(($VALIDATOR_WS_PORT + $idx))

    initial_container_configurations $LOCAL_CONTAINER_NAME

    launch_validator $LOCAL_CONTAINER_NAME "-p $LOCAL_PORT:30333 -p $LOCAL_RPC_PORT:9933 -p $LOCAL_WS_PORT:9944" "--rpc-cors all --rpc-methods=Unsafe --unsafe-rpc-external"
    generate_rotate_key $LOCAL_RPC_PORT
    delete_validator "$LOCAL_CONTAINER_NAME"
    launch_validator "$LOCAL_CONTAINER_NAME" "-p $LOCAL_PORT:30333 -p $LOCAL_RPC_PORT:9933 -p $LOCAL_WS_PORT:9944" ""
done

# Parachains

cp $PARACHAIN_SPEC_FILE $DATA_DIR/parachain-spec.json
cp $RELAY_CHAIN_SPEC_FILE $DATA_DIR/relay-chain-spec.json

sudo docker run \
    -v $DATA_DIR/parachain-spec.json:/zeitgeist/parachain-spec.json \
    --rm \
    $PARACHAIN_IMAGE \
    export-genesis-state \
    --chain /zeitgeist/parachain-spec.json \
    --parachain-id $PARACHAIN_ID > $DATA_DIR/zeitgeist-genesis-state

sudo docker run \
    -v $DATA_DIR/parachain-spec.json:/zeitgeist/parachain-spec.json \
    --rm \
    $PARACHAIN_IMAGE \
    export-genesis-wasm \
    --chain /zeitgeist/parachain-spec.json > $DATA_DIR/zeitgeist-genesis-wasm

for idx in $(seq 0 $PARACHAINS_NUM)
do
    LOCAL_CONTAINER_NAME="$PARACHAIN-$idx"
    LOCAL_PORT=$(($PARACHAIN_PORT + $idx))
    LOCAL_RPC_PORT=$(($PARACHAIN_RPC_PORT + $idx))
    LOCAL_WS_PORT=$(($PARACHAIN_WS_PORT + $idx))

    initial_container_configurations $LOCAL_CONTAINER_NAME

    sudo docker run \
        -d \
        -p $LOCAL_PORT:30333 \
        -p $LOCAL_RPC_PORT:9933 \
        -p $LOCAL_WS_PORT:9944 \
        -v $DATA_DIR/$LOCAL_CONTAINER_NAME:/zeitgeist/data \
        -v $DATA_DIR/parachain-spec.json:/zeitgeist/parachain-spec.json \
        -v $DATA_DIR/relay-chain-spec.json:/zeitgeist/relay-chain-spec.json \
        --name $LOCAL_CONTAINER_NAME \
        --restart always \
        $PARACHAIN_IMAGE \
        --base-path /zeitgeist/data \
        --chain /zeitgeist/parachain-spec.json \
        --collator \
        --parachain-id $PARACHAIN_ID \
        --rpc-cors all \
        --rpc-external \
        --ws-external \
        -- \
        --chain /zeitgeist/relay-chain-spec.json \
        --execution wasm
done
