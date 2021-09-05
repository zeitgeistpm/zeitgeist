#!/usr/bin/env bash

# Clippy with custom permissions

cargo clippy --all-features --release -- \
  -Dwarnings \
  -Aclippy::from_over_into \
  -Aclippy::let_and_return \
  -Aclippy::many_single_char_names \
  -Aclippy::too_many_arguments \
  -Aclippy::type_complexity \
  -Aclippy::unnecessary_cast \
  -Aclippy::unnecessary_mut_passed \
  -Aclippy::unused_unit \
  -Dclippy::integer_arithmetic