#!/usr/bin/env bash

set -euxo pipefail

VALIDATOR_KEY="0x57f8dc2f5ab09467896f47300f0424385e0621c4869aa60c02be9adcc98a0d1d"
ALICE_VALIDATOR="0x04d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"

SUDO_KEY="0x5c0d1176a568c1f92944340dbfed9e9c530ebca703c85910e7164cb7d1c9e47b"
ALICE_SUDO="0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"

BLOCK=$1
ID=battery_station_runtime_upgrade_test
PRODUCTION_IMAGE=zeitgeistpm/zeitgeist-node:latest

# Sync with live network
#
# Comment this whole section if `/tmp/migration` is already synced.

mkdir -p /tmp/migration
sudo docker run -d --name migration -v /tmp/migration:/zeitgeist/data $PRODUCTION_IMAGE --base-path /zeitgeist/data --chain battery_station --pruning archive
sleep 3m
sudo docker container stop migration
sudo docker container rm migration

# Run new node with latest changes

rm -rf /tmp/migration-copy
cp -r /tmp/migration /tmp/migration-copy
cargo build --bin zeitgeist --release
./target/release/zeitgeist export-state --base-path /tmp/migration-copy --chain battery_station --pruning archive $BLOCK > /tmp/test-upgrade.json

sed -i '/"bootNodes": \[/,/\]/c\ \ "bootNodes": [],' /tmp/test-upgrade.json
sed -i "s/\"chainType\": \"Live\"/\"chainType\": \"Local\"/" /tmp/test-upgrade.json
sed -i "s/\"id\": \".*\"/\"id\": \"$ID\"/" /tmp/test-upgrade.json
sed -i "s/\"name\": \".*\"/\"name\": \"Zeitgeist Battery Park Runtime Upgrade Test\"/" /tmp/test-upgrade.json
sed -i "s/\"protocolId\": \".*\"/\"protocolId\": \"$ID\"/" /tmp/test-upgrade.json
sed -i "s/\"$VALIDATOR_KEY\": \".*\"/\"$VALIDATOR_KEY\": \"$ALICE_VALIDATOR\"/" /tmp/test-upgrade.json
sed -i "s/\"$SUDO_KEY\": \".*\"/\"$SUDO_KEY\": \"$ALICE_SUDO\"/" /tmp/test-upgrade.json

./target/release/zeitgeist --base-path /tmp/migration-copy --chain /tmp/test-upgrade.json --alice