#!/usr/bin/env bash

# See `testing-network.sh`.

export BOOTNODES="--bootnodes /ip4/34.89.248.129/tcp/30333/p2p/12D3KooWD8CAZBgpeZiSVVbaj8mijR6mfgUsHNAmCKwsRoRnFod4 --bootnodes /ip4/35.242.217.240/tcp/30333/p2p/12D3KooWBthdCz4JshkMb4GxJXVwrHPv9GpWAgfh2hAdkyXQDKyN"
export DATA_DIR="$HOME/chachacha"
export DOCKER_POLKADOT_BIN="/usr/local/bin/polkadot"
export PARACHAINS_NUM="1"
export POLKADOT_IMAGE="centrifugeio/rococo:chachacha-v1"
export VALIDATOR_CHAIN="rococo-chachacha"
export VALIDATORS_NUM="2"

export PARACHAIN="zeitgeist-chachacha-parachain"
export PARACHAIN_PORT="30000"
export PARACHAIN_RPC_PORT="8000"
export PARACHAIN_WS_PORT="9000"

export VALIDATOR="zeitgeist-chachacha-validator"
export VALIDATOR_PORT="31000"
export VALIDATOR_RPC_PORT="8100"
export VALIDATOR_WS_PORT="9100"

export PARACHAIN_CHAIN="battery_park_staging"
export RELAY_CHAIN_SPEC_FILE="/tmp/relay-chain-spec.json"
curl -o $RELAY_CHAIN_SPEC_FILE https://storage.googleapis.com/centrifuge-artifact-releases/rococo-chachacha.json

. "$(dirname "$0")/testing-network.sh" --source-only