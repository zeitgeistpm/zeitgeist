#!/usr/bin/env bash

# Miscellaneous: Checks everything that is not fuzz, runtime or node related

set -euxo pipefail

. "$(dirname "$0")/aux-functions.sh" --source-only

cargo fmt --all -- --check

cargo clippy --all-features -- \
  -Dwarnings \
  -Aclippy::from_over_into \
  -Aclippy::let_and_return \
  -Aclippy::many_single_char_names \
  -Aclippy::too_many_arguments \
  -Aclippy::type_complexity \
  -Aclippy::unnecessary_cast \
  -Aclippy::unnecessary_mut_passed \
  -Aclippy::unused_unit

test_package_with_feature primitives default
test_package_with_feature primitives std

for package in zrml/*
do
  test_package_with_feature "$package" std
  test_package_with_feature "$package" std,runtime-benchmarks
done