#!/usr/bin/env bash

DATA_DIR="$HOME/rococo"
PARACHAIN_ID="9123"

CENTRIFUGE_POLKADOT_IMAGE="centrifugeio/rococo:chachacha-v1"
ZEITGEST_PARACHAIN_IMAGE="zeitgeistpm/zeitgeist-node-parachain"

CHACHACHA_VALIDATOR_0="chachacha-validator-0"
CHACHACHA_VALIDATOR_1="chachacha-validator-1"
ZEITGEST_PARACHAIN_0="zeitgeist-parachain-0"

sudo apt update
sudo apt install -y curl docker.io
sudo docker stop $CHACHACHA_VALIDATOR_0 $CHACHACHA_VALIDATOR_1 $ZEITGEST_PARACHAIN_0
sudo docker rm $CHACHACHA_VALIDATOR_0 $CHACHACHA_VALIDATOR_1 $ZEITGEST_PARACHAIN_0
mkdir -p $DATA_DIR/$CHACHACHA_VALIDATOR_0 $DATA_DIR/$CHACHACHA_VALIDATOR_1 $DATA_DIR/$ZEITGEST_PARACHAIN_0

# Functions

delete_validator() {
    local container_name=$1
    
    sudo docker stop $container_name
    sudo docker rm $container_name
}

generate_rotate_key() {
    local validator_port=$1
    
    sleep 2
    ROTATE_KEY=$(curl -H 'Content-Type:application/json;charset=utf-8' --data '{ "id":1, "jsonrpc":"2.0", "method":"author_rotateKeys" }' localhost:$validator_port)
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
        $CENTRIFUGE_POLKADOT_IMAGE \
        /usr/local/bin/polkadot \
        --bootnodes /ip4/34.89.248.129/tcp/30333/p2p/12D3KooWD8CAZBgpeZiSVVbaj8mijR6mfgUsHNAmCKwsRoRnFod4 \
        --bootnodes /ip4/35.242.217.240/tcp/30333/p2p/12D3KooWBthdCz4JshkMb4GxJXVwrHPv9GpWAgfh2hAdkyXQDKyN \
        --chain rococo-chachacha \
        --name $container_name \
        --validator \
        $validator_extra_params
}

# Validators

launch_validator "$CHACHACHA_VALIDATOR_0" "-p 30000:30333 -p 8000:9933 -p 9000:9944" "--rpc-cors all --rpc-methods=Unsafe --unsafe-rpc-external"
ROTATE_KEY_0=$(generate_rotate_key 8000)
delete_validator "$CHACHACHA_VALIDATOR_0"
launch_validator "$CHACHACHA_VALIDATOR_0" "-p 30000:30333 -p 8000:9933 -p 9000:9944" ""

launch_validator "$CHACHACHA_VALIDATOR_1" "-p 30001:30333 -p 8001:9933 -p 9001:9944" "--rpc-cors all --rpc-methods=Unsafe --unsafe-rpc-external"
ROTATE_KEY_1=$(generate_rotate_key 8001)
delete_validator "$CHACHACHA_VALIDATOR_1"
launch_validator "$CHACHACHA_VALIDATOR_1" "-p 30001:30333 -p 8001:9933 -p 9001:9944" ""

# Parachains

sudo docker run --rm $ZEITGEST_PARACHAIN_IMAGE export-genesis-state --parachain-id $PARACHAIN_ID > $DATA_DIR/zeitgeist-genesis-state
sudo docker run --rm $ZEITGEST_PARACHAIN_IMAGE export-genesis-wasm > $DATA_DIR/zeitgeist-genesis-wasm
curl -o $DATA_DIR/$ZEITGEST_PARACHAIN_0/rococo-chachacha.json https://storage.googleapis.com/centrifuge-artifact-releases/rococo-chachacha.json

sudo docker run \
    -d \
    -p 30200:30333 \
    -p 8100:9933 \
    -p 9100:9944 \
    -v $DATA_DIR/$ZEITGEST_PARACHAIN_0:/zeitgeist/data \
    --name $ZEITGEST_PARACHAIN_0 \
    --restart always \
    $ZEITGEST_PARACHAIN_IMAGE \
    --collator \
    --parachain-id $PARACHAIN_ID \
    --rpc-cors all \
    --rpc-external \
    --ws-external \
    -- \
    --chain /zeitgeist/data/rococo-chachacha.json \
    --execution wasm

echo $ROTATE_KEY_0
echo $ROTATE_KEY_1
