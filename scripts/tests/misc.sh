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

test_package_with_feature primitives default $rustflags
test_package_with_feature primitives std $rustflags

no_runtime_benchmarks=('court' 'market-commons' 'rikiddo')

cargo test --package zeitgeist-runtime --lib -- --nocapture

cargo test -p zrml-prediction-markets --features parachain


for package in zrml/*
do
  test_package_with_feature "$package" std "$rustflags"
  echo "TEST $package std"

  if [[ " ${no_runtime_benchmarks[*]} " != *" ${package##*/} "* ]]; then
    test_package_with_feature "$package" std,runtime-benchmarks "$rustflags"
    echo "TEST $package std,runtime-benchmarks"
  fi
done

if [[ ! -z "$rustflags" ]]; then
    grcov . --binary-path ./target/debug/deps/ -s . -t lcov --branch --ignore-not-existing --llvm --ignore '../*' --ignore "/*" -o $RUNNER_TEMP/zeitgeist-test-coverage.lcov
fi
