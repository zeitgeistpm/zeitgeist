#!/bin/bash

# Exit on any error
set -e

cd ../

cargo build --features parachain

cd integration-tests/