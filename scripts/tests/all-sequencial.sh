#!/usr/bin/env bash

# Checks everything in a sequential fashion. Useful for debugging but slow to compile/complete.
#
# IMPORTANT: CI verifies most of the following scripts in parallel

. "$(dirname "$0")/clippy.sh" --source-only
. "$(dirname "$0")/standalone.sh" --source-only
. "$(dirname "$0")/parachain.sh" --source-only
. "$(dirname "$0")/test_standalone.sh" --source-only
. "$(dirname "$0")/test_parachain.sh" --source-only
. "$(dirname "$0")/fuzz.sh" --source-only
