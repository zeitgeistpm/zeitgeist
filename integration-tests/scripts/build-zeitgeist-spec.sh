#!/bin/bash

# Exit on any error
set -e

# Always run the commands from the "integration-tests" dir
cd $(dirname $0)/..

mkdir -p specs
../target/debug/zeitgeist build-spec --disable-default-bootnode --add-bootnode "/ip4/127.0.0.1/tcp/33049/ws/p2p/12D3KooWHVMhQDHBpj9vQmssgyfspYecgV6e3hH1dQVDUkUbCYC9" --parachain-id 2101 --raw > specs/zeitgeist-parachain-2101.json