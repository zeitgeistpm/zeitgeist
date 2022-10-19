#!/usr/bin/env bash

# Takes a staging network chain to create a new production-ready specification

set -euxo pipefail

# For example, node/res/bs_parachain.json
OUTPUT_FILE=
# For example, "Battery Station"
PROD_CHAIN_NAME=
# For example, battery_station
PROD_CHAIN_PROTOCOL_ID=
# For example, battery_station_mainnet
PROD_CHAIN_ID=
# For example, battery_station_staging
STAGE_CHAIN=

cargo build --profile=production --bin zeitgeist --features parachain
./target/production/zeitgeist build-spec --chain $STAGE_CHAIN --disable-default-bootnode > $OUTPUT_FILE

sed -i "s/\"id\": \".*\"/\"id\": \"$PROD_CHAIN_ID\"/" $OUTPUT_FILE
sed -i "s/\"name\": \".*\"/\"name\": \"$PROD_CHAIN_NAME\"/" $OUTPUT_FILE
sed -i "s/\"protocolId\": \".*\"/\"protocolId\": \"$PROD_CHAIN_PROTOCOL_ID\"/" $OUTPUT_FILE

./target/production/zeitgeist build-spec --chain $OUTPUT_FILE --disable-default-bootnode --raw > $OUTPUT_FILE.raw
rm -f $OUTPUT_FILE
mv $OUTPUT_FILE.raw $OUTPUT_FILE