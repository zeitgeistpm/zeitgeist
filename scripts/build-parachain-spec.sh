#!/usr/bin/env bash

# Takes a staging network chain to create a new production-ready specification

set -euxo pipefail

# For example, node/res/bs_parachain.json
OUTPUT_FILE=
# For example, battery_station
PROD_CHAIN=
# For example, battery_station_staging
STAGE_CHAIN=

cargo build --bin zeitgeist --features parachain --release 
./target/release/zeitgeist build-spec --chain $STAGE_CHAIN > $OUTPUT_FILE

sed -i "s/\"id\": \".*\"/\"id\": \"$PROD_CHAIN\"/" $OUTPUT_FILE
sed -i "s/\"name\": \".*\"/\"name\": \"Zeitgeist - $PROD_CHAIN\"/" $OUTPUT_FILE
sed -i "s/\"protocolId\": \".*\"/\"protocolId\": \"$PROD_CHAIN\"/" $OUTPUT_FILE

./target/release/zeitgeist build-spec --chain $OUTPUT_FILE --raw > $OUTPUT_FILE.raw
rm -f $OUTPUT_FILE
mv $OUTPUT_FILE.raw $OUTPUT_FILE