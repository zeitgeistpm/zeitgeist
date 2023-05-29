#!/usr/bin/env bash

# Miscellaneous: Checks everything that is not clippy, fuzz, runtime or node related

set -euxo pipefail

. "$(dirname "$0")/aux-functions.sh" --source-only

if [ -z "${1:-}" ]
then
    rustflags=""
else
    rustflags=$1
fi

# Test standalone
RUSTFLAGS="${rustflags#-Cinstrument-coverage}" cargo test --features=default,runtime-benchmarks

# Test parachain
RUSTFLAGS="${rustflags#-Cinstrument-coverage}" cargo test --features=default,parachain,runtime-benchmarks

if [[ $rustflags == *"-Cinstrument-coverage"* ]]; then
  test_package_with_feature primitives std "$rustflags"
  no_runtime_benchmarks=('court' 'market-commons' 'rikiddo')

  for package in zrml/*; do
    if [[ " ${no_runtime_benchmarks[*]} " != *" ${package##*/} "* ]]; then
      echo "TEST $package std,runtime-benchmarks"
      test_package_with_feature "$package" std,runtime-benchmarks "$rustflags"
    else
      echo "TEST $package std"
      test_package_with_feature "$package" std "$rustflags"
    fi
  done

  grcov . --binary-path ./target/debug/deps/ -s . -t lcov --branch --ignore-not-existing --llvm --ignore '../*' --ignore "/*" -o $RUNNER_TEMP/zeitgeist-test-coverage.lcov
fi