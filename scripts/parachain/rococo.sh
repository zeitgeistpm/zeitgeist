#!/usr/bin/env bash

# See `testing-network.sh`.

export BOOTNODES=" "
export DOCKER_POLKADOT_BIN=" "
export RELAY_CHAIN_SPEC_FILE="/tmp/relay-chain-spec.json"
export VALIDATOR_CHAIN="rococo"

export POLKADOT_IMAGE="parity/rococo:rococo-v1-0.8.30-943038a8-f14fa75f"

export PARACHAIN_0="zeitgeist-rococo-parachain-0"
export PARACHAIN_0_PORT=32000
export PARACHAIN_0_RPC_PORT=8200
export PARACHAIN_0_WS_PORT=9200

export VALIDATOR_0="zeitgeist-rococo-validator-0"
export VALIDATOR_0_PORT=33000
export VALIDATOR_0_RPC_PORT=8300
export VALIDATOR_0_WS_PORT=9300

export VALIDATOR_1="zeitgeist-rococo-validator-1"
export VALIDATOR_1_PORT=33001
export VALIDATOR_1_RPC_PORT=8301
export VALIDATOR_1_WS_PORT=9301

curl -o $RELAY_CHAIN_SPEC_FILE https://raw.githubusercontent.com/paritytech/polkadot/943038a888bfaf736142642e2610b248f7af486c/node/service/res/rococo.json

. "$(dirname "$0")/testing-network.sh" --source-only
