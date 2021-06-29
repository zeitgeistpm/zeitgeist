#!/usr/bin/env bash

# Takes a staging network chain to create a new production-ready specification

PROD_CHAIN="battery_station"
STAGE_CHAIN="battery_station_staging"
OUTPUT_FILE="node/res/bs_parachain.json"

cargo run --bin zeitgeist --features parachain --release -- build-spec --chain $STAGE_CHAIN --disable-default-bootnode > $OUTPUT_FILE

sed -i "s/\"id\": \".*\"/\"id\": \"$PROD_CHAIN\"/" $OUTPUT_FILE
sed -i "s/\"name\": \".*\"/\"name\": \"Zeitgeist - $PROD_CHAIN\"/" $OUTPUT_FILE
sed -i "s/\"protocolId\": \".*\"/\"protocolId\": \"$PROD_CHAIN\"/" $OUTPUT_FILE