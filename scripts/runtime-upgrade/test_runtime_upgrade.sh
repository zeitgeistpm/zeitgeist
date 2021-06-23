
block=$1

./target/release/zeitgeist export-state $block --chain battery_park --pruning archive > test-upgrade.json
node scripts/runtime-upgrade/adjustTestUpgradeJson.js
./target/release/zeitgeist purge-chain --chain ./test-upgrade.json -y
./target/release/zeitgeist --chain ./test-upgrade.json --alice