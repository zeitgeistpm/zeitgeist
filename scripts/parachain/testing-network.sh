#!/usr/bin/env bash

# Not meant to be called directly.
#
# Common script used by live testing network (Rococo, Chachacha, etc) scipts. Spin-ups
# $VALIDATORS_NUM validators and $PARACHAINS_NUM parachains using Docker.
#
# Only required variable is "RELAY_CHAIN_SPEC_FILE", everything else defaults to common
# values if not specified.
#
# First parachain will always be a non-validator node while the remaining nodes will be validators

set -euxo pipefail

# Variables

PARACHAIN_IMAGE="${PARACHAIN_IMAGE:-zeitgeistpm/zeitgeist-node-parachain:sha-f1daa31}"
# Minus one because `seq` is inclusive
PARACHAINS_NUM=$(($PARACHAINS_NUM - 1))
# Minus one because `seq` is inclusive
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
if [ $VALIDATORS_NUM -gt 0 ]; then
    sudo docker pull $POLKADOT_IMAGE
fi

sudo docker container stop $(sudo docker container ls -aq --filter name=$PARACHAIN*) &> /dev/null || true
sudo docker container stop $(sudo docker container ls -aq --filter name=$VALIDATOR*) &> /dev/null || true

sudo docker container rm $(sudo docker container ls -aq --filter name=$PARACHAIN*) &> /dev/null || true
sudo docker container rm $(sudo docker container ls -aq --filter name=$VALIDATOR*) &> /dev/null || true

mkdir -p $DATA_DIR

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

cp $RELAY_CHAIN_SPEC_FILE $DATA_DIR/relay-chain-spec.json

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

launch_parachain() {
    local parachain_idx=$1
    local parachain_extra_params=$2

    LOCAL_CONTAINER_NAME="$PARACHAIN-$parachain_idx"
    LOCAL_PORT=$(($PARACHAIN_PORT + $parachain_idx))
    LOCAL_RPC_PORT=$(($PARACHAIN_RPC_PORT + $parachain_idx))
    LOCAL_WS_PORT=$(($PARACHAIN_WS_PORT + $parachain_idx))

    initial_container_configurations $LOCAL_CONTAINER_NAME

    sudo docker run \
        -d \
        -p $LOCAL_PORT:30333 \
        -p $LOCAL_RPC_PORT:9933 \
        -p $LOCAL_WS_PORT:9944 \
        -v $DATA_DIR/$LOCAL_CONTAINER_NAME:/zeitgeist/data \
        -v $DATA_DIR/relay-chain-spec.json:/zeitgeist/relay-chain-spec.json \
        --name $LOCAL_CONTAINER_NAME \
        --restart always \
        $PARACHAIN_IMAGE \
        --base-path /zeitgeist/data \
        --chain $PARACHAIN_CHAIN \
        --parachain-id $PARACHAIN_ID \
        $parachain_extra_params \
        -- \
        --chain /zeitgeist/relay-chain-spec.json \
        --execution wasm
}

if [[ "$PARACHAINS_NUM" -ge 0 ]];
then
    launch_parachain "0" "--rpc-cors all --rpc-external --ws-external"

    for idx in $(seq 1 $PARACHAINS_NUM)
    do
        launch_parachain "$idx" "--collator"
    done
fi