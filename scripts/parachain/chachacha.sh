#!/usr/bin/env bash

# See `testing-network.sh`.

export RELAY_CHAIN_SPEC_FILE="/tmp/relay-chain-spec.json"

curl -o $RELAY_CHAIN_SPEC_FILE https://storage.googleapis.com/centrifuge-artifact-releases/rococo-chachacha.json

. "$(dirname "$0")/testing-network.sh" --source-only
