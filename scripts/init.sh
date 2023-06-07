#!/usr/bin/env bash

set -e

echo "*** Initializing build environment"

# No sudo during docker build
if [ -f /.dockerenv ]; then
   apt-get update && \
   apt-get install -y build-essential clang curl libssl-dev protobuf-compiler
else
   sudo apt-get update && \
   sudo apt-get install -y build-essential clang curl libssl-dev protobuf-compiler
fi

curl https://sh.rustup.rs -sSf | sh -s -- -y && \
   . "$HOME/.cargo/env" && \
   rustup show
