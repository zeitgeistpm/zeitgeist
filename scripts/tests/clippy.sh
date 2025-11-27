#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later

cargo clippy --all-features --all-targets -- -Dwarnings -Aclippy::manual-inspect -Aclippy::useless_conversion -Aclippy::result-large-err
