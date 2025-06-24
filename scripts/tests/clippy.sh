#!/usr/bin/env bash

cargo clippy --all-features --all-targets -- -Dwarnings -Aclippy::manual-inspect -Aclippy::useless_conversion -Aclippy::result-large-err
