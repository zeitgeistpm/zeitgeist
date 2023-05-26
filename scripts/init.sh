#!/usr/bin/env bash

set -e

echo "*** Initializing build environment"

sudo apt-get update && \
   sudo apt-get install -y build-essential clang curl libssl-dev protobuf-compiler

curl https://sh.rustup.rs -sSf | sh -s -- -y && \
   source "$HOME/.cargo/env" && \
   rustup show
