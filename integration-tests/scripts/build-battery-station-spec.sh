#!/bin/bash

# Exit on any error
set -e

# Always run the commands from the "integration-tests" dir
cd $(dirname $0)/..

mkdir -p specs
../target/release/zeitgeist build-spec --chain=dev --raw > specs/battery-station-parachain-2101.json