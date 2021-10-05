#!/usr/bin/env bash

set -euxo pipefail

# ***** Secrets *****
#
# Do not commit any secret
#
export OVERALL_SECRET=""
export PARACHAIN_FIRST_BOOTNODE_NODE_KEY=""
export PARACHAIN_NIMBUS_SEED=""
export VALIDATOR_FIRST_BOOTNODE_NODE_KEY=""
export VALIDATOR_SECOND_BOOTNODE_NODE_KEY=""
#
# Do not commit any secret
#
# ***** Secrets *****

export DATA_DIR="$HOME/battery-station-relay"
export DOCKER_POLKADOT_BIN="/usr/local/bin/polkadot"

export PARACHAIN_CHAIN="battery_station"
export PARACHAIN_FIRST_BOOTNODE_ADDR="/ip4/45.33.117.205/tcp/30001/p2p/12D3KooWBMSGsvMa2A7A9PA2CptRFg9UFaWmNgcaXRxr1pE1jbe9"
export PARACHAIN_ID="2050"
export PARACHAIN_IMAGE="zeitgeistpm/zeitgeist-node-parachain:sha-74486f3"
export PARACHAIN_NIMBUS_PK="0xe6ea0b63b2b5b7247a1e8280350a14c5f9e7745dec2fe3428b68aa4167d48e66"
export PARACHAIN_PORT="30000"
export PARACHAIN_RPC_PORT="8000"
export PARACHAIN_WS_PORT="9000"
export PARACHAIN="zeitgeist-battery-station-relay-parachain"

export VALIDATOR_FIRST_BOOTNODE_ADDR="/ip4/45.33.117.205/tcp/31001/p2p/12D3KooWHgbvdWFwNQiUPbqncwPmGCHKE8gUQLbzbCzaVbkJ1crJ"
export VALIDATOR_IMAGE="zeitgeistpm/zeitgeist-relay-chain:sha-f116c7a"
export VALIDATOR_PORT="31000"
export VALIDATOR_RPC_PORT="8100"
export VALIDATOR_SECOND_BOOTNODE_ADDR="/ip4/45.33.117.205/tcp/31002/p2p/12D3KooWE5KxMrfJLWCpaJmAPLWDm9rS612VcZg2JP6AYgxrGuuE"
export VALIDATOR_WS_PORT="9100"
export VALIDATOR="zeitgeist-battery-station-relay-validator"

export RELAY_CHAIN_SPEC_FILE="/tmp/relay-chain-spec.json"
curl -o $RELAY_CHAIN_SPEC_FILE https://raw.githubusercontent.com/zeitgeistpm/polkadot/battery-station-relay/node/service/res/battery-station-relay.json

. "$(dirname "$0")/testing-network-commons.sh" --source-only
