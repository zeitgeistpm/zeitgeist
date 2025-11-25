#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later

source ./scripts/build-node-configuration.sh

pushd ..

if [ -z "$CI" ]; then
    cargo build --profile=$PROFILE --features=$FEATURES --bin=zeitgeist
fi

popd