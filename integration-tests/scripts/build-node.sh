#!/usr/bin/env bash

if [ ! -d "./integration-tests/scripts" ]; then
    echo "Please execute this script from the root of the Zeitgeist project folder"
    exit 1
fi;

source ./integration-tests/scripts/build-node-configuration.sh

if [ -z "$CI" ]; then
    cargo build --profile=$PROFILE --features=$FEATURES --bin=zeitgeist
fi
