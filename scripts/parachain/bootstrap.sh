#!/usr/bin/env bash

set -euxo pipefail

# ***** Secrets *****
#
# Do not commit any secret
#
# Use https://github.com/paritytech/polkadot/blob/master/scripts/prepare-test-net.sh
# and the subkey tool (either standalone or included as "key" command in zeitgeist binary)
# to generate the validator keys, node key and the parachain collator nimbus key.
# Don't forget to add them to the genesis storage.
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

export PARACHAIN_CHAIN="battery_station"
export PARACHAIN_FIRST_BOOTNODE_ADDR="/ip4/127.0.0.1/tcp/30001/p2p/12D3KooWBMSGsvMa2A7A9PA2CptRFg9UFaWmNgcaXRxr1pE1jbe9"
export PARACHAIN_ID="2050"
export PARACHAIN_IMAGE="zeitgeistpm/zeitgeist-node-parachain:latest"
export PARACHAIN_NIMBUS_PK="0xe6ea0b63b2b5b7247a1e8280350a14c5f9e7745dec2fe3428b68aa4167d48e66"
export PARACHAIN_PORT="30000"
export PARACHAIN_RPC_PORT="8000"
export PARACHAIN_WS_PORT="9000"
export PARACHAIN_RELAY_PORT="30300"
export PARACHAIN_RELAY_RPC_PORT="8300"
export PARACHAIN_RELAY_WS_PORT="9300"
export PARACHAIN="battery-station-parachain"

export VALIDATOR_CHAIN="battery_station_relay" 
export VALIDATOR_FIRST_BOOTNODE_ADDR="/ip4/127.0.0.1/tcp/30601/p2p/12D3KooWBn6wRSVFW3ir2pVBg1TGbG4utihTS479T8uVEsiZSnBd"
export VALIDATOR_IMAGE="zeitgeistpm/zeitgeist-relay-chain:latest"
export VALIDATOR_PORT="30600"
export VALIDATOR_RPC_PORT="8600"
export VALIDATOR_SECOND_BOOTNODE_ADDR="/ip4/127.0.0.1/tcp/30602/p2p/12D3KooWMt1dB8xqLZn71vCTR5qNsdGiWnM38vSSyS4BmCrxSUJ7"
export VALIDATOR_WS_PORT="9600"
export VALIDATOR="battery-station-relaychain"

. "$(dirname "$0")/testing-network-commons.sh" --source-only
