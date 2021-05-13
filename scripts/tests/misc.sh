#!/usr/bin/env bash

# Miscellaneous: Checks everything that is not runtime or node related

set -euxo pipefail

. "$(dirname "$0")/aux-functions.sh" --source-only

cargo fmt --all -- --check

test_package_with_feature primitives default
test_package_with_feature primitives std

for package in zrml/*
do
  test_package_with_feature "$package" std
  test_package_with_feature "$package" std,runtime-benchmarks
done