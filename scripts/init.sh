#!/usr/bin/env bash

# Initializes a build environment.
# Passing one argument results in avoiding the usage of sudo for privileged commands.

set -e

echo "*** Initializing build environment"

if [ -n "$1" ]; then
   apt-get update && \
   apt-get install -y build-essential clang curl libssl-dev protobuf-compiler
else
   sudo apt-get update && \
   sudo apt-get install -y build-essential clang curl libssl-dev protobuf-compiler
fi

curl https://sh.rustup.rs -sSf | sh -s -- -y && \
   . "$HOME/.cargo/env" && \
   rustup show
