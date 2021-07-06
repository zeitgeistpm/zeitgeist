#!/usr/bin/env bash

# See `testing-network.sh`.

export BOOTNODES=" "
export DATA_DIR="$HOME/rococo"
export DOCKER_POLKADOT_BIN=" "
export PARACHAINS_NUM="2"
export POLKADOT_IMAGE="parity/polkadot:v0.9.6"
export VALIDATOR_CHAIN="rococo"
export VALIDATORS_NUM="0"

export PARACHAIN="zeitgeist-rococo-parachain"
export PARACHAIN_PORT="32000"
export PARACHAIN_RPC_PORT="8200"
export PARACHAIN_WS_PORT="9200"

export VALIDATOR="zeitgeist-rococo-validator"
export VALIDATOR_PORT="33000"
export VALIDATOR_RPC_PORT="8300"
export VALIDATOR_WS_PORT="9300"

export PARACHAIN_CHAIN="battery_station_staging"
export RELAY_CHAIN_SPEC_FILE="/tmp/relay-chain-spec.json"
curl -o $RELAY_CHAIN_SPEC_FILE https://raw.githubusercontent.com/paritytech/polkadot/release-v0.9.6/node/service/res/rococo.json

. "$(dirname "$0")/testing-network.sh" --source-only