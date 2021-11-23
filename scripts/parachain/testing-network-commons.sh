#!/usr/bin/env bash

# Not meant to be called directly.
#
# Common script used by live testing networks (Rococo, Chachacha, Battery Station Relay, etc). Spin-ups
# 3 validators and 2 parachains using Docker.
#
# First parachain and relay chain nodes will always be a light node while the remaining nodes will be actual validators.

# Generic Functions

delete_container() {
    local container_name=$1

    sudo docker stop $container_name
    sudo docker rm $container_name
}

generate_account_id() {
    sudo docker run --rm $VALIDATOR_IMAGE key inspect ${3:-} "$OVERALL_SECRET//$1//$2" | grep "Account ID" | awk '{ print $3 }'
}

generate_author_insertKey_with_account_id() {
    ACCOUNT=$(generate_account_id $1 $2 "$3")
    SEED=$OVERALL_SECRET//$1//$2

    printf '{"jsonrpc":"2.0","id":1,"method":"author_insertKey","params":["'"$4"'","'"$SEED"'","'"$ACCOUNT"'"]}'
}

generate_author_insertKey_with_public_key() {
    PUBLIC_KEY=$(generate_public_key $1 $2 "$3")
    SEED=$OVERALL_SECRET//$1//$2

    printf '{"jsonrpc":"2.0","id":1,"method":"author_insertKey","params":["'"$4"'","'"$SEED"'","'"$PUBLIC_KEY"'"]}'
}

generate_public_key() {
    sudo docker run --rm $VALIDATOR_IMAGE key inspect ${3:-} "$OVERALL_SECRET//$1//$2" | grep "Public key (hex)" | awk '{ print $4 }'
}

initial_container_configurations() {
    local container_name=$1

    mkdir -p $DATA_DIR/$container_name
}

# Init

sudo apt update
sudo apt install -y curl docker.io
sudo docker pull $PARACHAIN_IMAGE
sudo docker pull $VALIDATOR_IMAGE

sudo docker container stop $(sudo docker container ls -aq --filter name=$PARACHAIN*) &> /dev/null || true
sudo docker container stop $(sudo docker container ls -aq --filter name=$VALIDATOR*) &> /dev/null || true

sudo docker container rm $(sudo docker container ls -aq --filter name=$PARACHAIN*) &> /dev/null || true
sudo docker container rm $(sudo docker container ls -aq --filter name=$VALIDATOR*) &> /dev/null || true

mkdir -p $DATA_DIR

cp $RELAY_CHAIN_SPEC_FILE $DATA_DIR/relay-chain-spec.json

# Validators

inject_keys() {
    local idx=$1
    local rpc_port=$2

    curl http://localhost:$rpc_port -H "Content-Type:application/json;charset=utf-8" -d "$(generate_author_insertKey_with_account_id $idx babe '--scheme sr25519' babe)"
    curl http://localhost:$rpc_port -H "Content-Type:application/json;charset=utf-8" -d "$(generate_author_insertKey_with_account_id $idx grandpa '--scheme ed25519' gran)"
    curl http://localhost:$rpc_port -H "Content-Type:application/json;charset=utf-8" -d "$(generate_author_insertKey_with_account_id $idx im_online '--scheme sr25519' imon)"
    curl http://localhost:$rpc_port -H "Content-Type:application/json;charset=utf-8" -d "$(generate_author_insertKey_with_account_id $idx para_validator '--scheme sr25519' para)"
    curl http://localhost:$rpc_port -H "Content-Type:application/json;charset=utf-8" -d "$(generate_author_insertKey_with_account_id $idx para_assignment '--scheme sr25519' asgn)"
    curl http://localhost:$rpc_port -H "Content-Type:application/json;charset=utf-8" -d "$(generate_author_insertKey_with_account_id $idx authority_discovery '--scheme sr25519' audi)"
    curl http://localhost:$rpc_port -H "Content-Type:application/json;charset=utf-8" -d "$(generate_author_insertKey_with_public_key $idx beefy '--scheme ecdsa' beef)"
}

launch_validator() {
    local container_name=$1
    local validator_extra_params=$2

    sudo docker run \
        -d \
        -v $DATA_DIR/relay-chain-spec.json:/zeitgeist/relay-chain-spec.json \
        -v $DATA_DIR/$container_name:/data \
        --name=$container_name \
        --network=host \
        --restart=always \
        $VALIDATOR_IMAGE \
        --base-path=/data \
        --chain=/zeitgeist/relay-chain-spec.json \
        --name=$container_name \
        $validator_extra_params
}

launch_configured_validator() {
    local idx=$1
    
    local container_name="$VALIDATOR-$idx"
    local relay_port=$(($VALIDATOR_PORT + $idx))
    local relay_rpc_port=$(($VALIDATOR_RPC_PORT + $idx))
    local relay_ws_port=$(($VALIDATOR_WS_PORT + $idx))
    local relay_port_extra="--port=$relay_port --rpc-port=$relay_rpc_port --ws-port=$relay_ws_port"

    initial_container_configurations "$container_name"

    if (( $idx <= 0 ))
    then
        launch_validator "$container_name" "--bootnodes=$VALIDATOR_FIRST_BOOTNODE_ADDR --bootnodes=$VALIDATOR_SECOND_BOOTNODE_ADDR --pruning archive --rpc-cors=all --rpc-external --ws-external $relay_port_extra"
    else
        launch_validator  "--rpc-cors=all --rpc-methods=Unsafe --unsafe-rpc-external --validator $relay_port_extra"
        sleep 10
        inject_keys $idx $relay_rpc_port
        delete_container "$container_name"

        if (( $idx == 1 ))
        then
            launch_validator "$container_name" "--bootnodes=$VALIDATOR_SECOND_BOOTNODE_ADDR --node-key=$VALIDATOR_FIRST_BOOTNODE_NODE_KEY --validator $relay_port_extra"
        elif (( $idx == 2 ))
        then
            launch_validator "$container_name" "--bootnodes=$VALIDATOR_FIRST_BOOTNODE_ADDR --node-key=$VALIDATOR_SECOND_BOOTNODE_NODE_KEY --validator $relay_port_extra"
        else
            launch_validator "$container_name" "--bootnodes=$VALIDATOR_FIRST_BOOTNODE_ADDR --bootnodes=$VALIDATOR_SECOND_BOOTNODE_ADDR --validator $relay_port_extra"
        fi
    fi
}

launch_configured_validator 0
launch_configured_validator 1
launch_configured_validator 2

# Parachains

sudo docker run \
    --rm \
    $PARACHAIN_IMAGE \
    export-genesis-state \
    --chain=$PARACHAIN_CHAIN \
    --parachain-id=$PARACHAIN_ID > $DATA_DIR/zeitgeist-genesis-state

sudo docker run \
    --rm \
    $PARACHAIN_IMAGE \
    export-genesis-wasm \
    --chain=$PARACHAIN_CHAIN > $DATA_DIR/zeitgeist-genesis-wasm

launch_parachain() {
    local container_name=$1
    local parachain_extra_params=$2
    local relaychain_extra_params=$3

    sudo docker run \
        -d \
        -v $DATA_DIR/$container_name:/zeitgeist/data \
        -v $DATA_DIR/relay-chain-spec.json:/zeitgeist/relay-chain-spec.json \
        --name=$container_name \
        --restart=always \
        --network=host \
        $PARACHAIN_IMAGE \
        --base-path=/zeitgeist/data \
        --chain=$PARACHAIN_CHAIN \
        --parachain-id=$PARACHAIN_ID \
        $parachain_extra_params \
        -- \
        --bootnodes=$VALIDATOR_FIRST_BOOTNODE_ADDR \
        --bootnodes=$VALIDATOR_SECOND_BOOTNODE_ADDR \
        --chain=/zeitgeist/relay-chain-spec.json \
        --execution=wasm \
        $relaychain_extra_params
}

launch_configured_parachain() {
    local idx=$1
    local nimbus_seed=$2
    local nimbus_pk=$3

    local container_name="$PARACHAIN-$idx"
    local para_port=$(($PARACHAIN_PORT + $idx))
    local para_rpc_port=$(($PARACHAIN_RPC_PORT + $idx))
    local para_ws_port=$(($PARACHAIN_WS_PORT + $idx))
    local relay_port=$(($PARACHAIN_RELAY_PORT + $idx))
    local relay_rpc_port=$(($PARACHAIN_RELAY_RPC_PORT + $idx))
    local relay_ws_port=$(($PARACHAIN_RELAY_WS_PORT + $idx))
    local para_port_extra="--port=$para_port --rpc-port=$para_rpc_port --ws-port=$para_ws_port"
    local relay_port_extra="--port=$relay_port --rpc-port=$relay_rpc_port --ws-port=$relay_ws_port"

    initial_container_configurations "$container_name"

    if (( $idx <= 0 ))
    then
        launch_parachain "$container_name" "--bootnodes=$PARACHAIN_FIRST_BOOTNODE_ADDR --pruning=archive --rpc-cors=all --rpc-external --ws-external $para_port_extra" "$relay_port_extra"
    else
        launch_parachain "$container_name" "--collator --rpc-cors=all --rpc-methods=Unsafe --unsafe-rpc-external $para_port_extra" "$relay_port_extra"
        sleep 10
        DATA='{ "id":1, "jsonrpc":"2.0", "method":"author_insertKey", "params":["nmbs", "'"$nimbus_seed"'", "'"$nimbus_pk"'"] }'
        curl -H 'Content-Type: application/json' --data "$DATA" localhost:$para_rpc_port
        delete_container "$container_name"

        if (( $idx == 1 ))
        then
            launch_parachain "$container_name" "--collator --node-key=$PARACHAIN_FIRST_BOOTNODE_NODE_KEY $para_port_extra" "$relay_port_extra"
        else
            launch_parachain "$container_name" "--bootnodes=$PARACHAIN_FIRST_BOOTNODE_ADDR --collator $para_port_extra" "$relay_port_extra"
        fi
    fi  
}

launch_configured_parachain 0 "" ""
launch_configured_parachain 1 "$PARACHAIN_NIMBUS_SEED" "$PARACHAIN_NIMBUS_PK"