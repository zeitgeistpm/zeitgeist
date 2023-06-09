#!/usr/bin/env bash

# Coverage: Generate project coverage files using grcov 

set -euxo pipefail

. "$(dirname "$0")/aux-functions.sh" --source-only

export CARGO_INCREMENTAL=0
export LLVM_PROFILE_FILE="cargo-test-%p-%m.profraw"
rustflags="-Cinstrument-coverage"

test_package_with_feature "primitives" "std" "$rustflags"
no_runtime_benchmarks=('court' 'market-commons' 'rikiddo')

for package in zrml/*; do
if [[ " ${no_runtime_benchmarks[*]} " != *" ${package##*/} "* ]]; then
    echo "TEST $package std,runtime-benchmarks"
    test_package_with_feature "$package" "std,runtime-benchmarks" "$rustflags"
else
    echo "TEST $package std"
    test_package_with_feature "$package" "std" "$rustflags"
fi
done

unset CARGO_INCREMENTAL LLVM_PROFILE_FILE

grcov . --binary-path ./target/debug/deps/ -s . -t lcov --branch --ignore-not-existing --llvm --ignore '../*' --ignore "/*" -o $RUNNER_TEMP/zeitgeist-test-coverage.lcov