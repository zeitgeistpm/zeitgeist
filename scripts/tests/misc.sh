#!/usr/bin/env bash

# Miscellaneous: Checks everything that is not clippy, fuzz, runtime or node related

set -euxo pipefail

. "$(dirname "$0")/aux-functions.sh" --source-only

test_package_with_feature primitives default
test_package_with_feature primitives std

no_runtime_benchmarks=('court' 'market-commons' 'rikiddo')

cargo test --package zeitgeist-runtime --lib -- --nocapture

for package in zrml/*
do
  test_package_with_feature "$package" std
  echo "TEST $package std"

  if [[ " ${no_runtime_benchmarks[*]} " != *" ${package##*/} "* ]]; then
    test_package_with_feature "$package" std,runtime-benchmarks
    echo "TEST $package std,runtime-benchmarks"
  fi
done

grcov . --binary-path ./target/release/deps/ -s . -t lcov --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o $RUNNER_TEMP/zeitgeist-test-coverage.lcov
