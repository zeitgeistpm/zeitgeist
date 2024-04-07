#!/usr/bin/env bash

source ./scripts/build-node-configuration.sh

pushd ..

if [ -z "$CI" ]; then
    cargo build --profile=$PROFILE --features=$FEATURES --bin=zeitgeist
fi

popd