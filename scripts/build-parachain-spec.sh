#!/usr/bin/env bash

# Takes a staging network chain to create a new production-ready specification

PROD_CHAIN="battery_park"
STAGE_CHAIN="battery_park_staging"

cargo run --bin zeitgeist --features parachain --release -- build-spec --chain $STAGE_CHAIN --disable-default-bootnode > node/res/bp_parachain.json

sed -i "s/\"id\": \".*\"/\"id\": \"$PROD_CHAIN\"/" node/res/bp_parachain.json
sed -i "s/\"name\": \".*\"/\"name\": \"Zeitgeist - $PROD_CHAIN\"/" node/res/bp_parachain.json
sed -i "s/\"protocolId\": \".*\"/\"protocolId\": \"$PROD_CHAIN\"/" node/res/bp_parachain.json