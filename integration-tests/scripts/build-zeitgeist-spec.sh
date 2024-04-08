#!/bin/bash

# Exit on any error
set -e

# Always run the commands from the "integration-tests" dir
cd $(dirname $0)/..

mkdir -p specs
../target/release/zeitgeist build-spec --chain=zeitgeist --raw > specs/zeitgeist-parachain-2092.json