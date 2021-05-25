#!/usr/bin/env bash

# Checks everything in a sequential fashion. Useful for debugging but slow to compile/complete.
#
# IMPORTANT: CI verifies all of the following scripts in parallel

. "$(dirname "$0")/misc.sh" --source-only
. "$(dirname "$0")/parachain.sh" --source-only
. "$(dirname "$0")/standalone.sh" --source-only
. "$(dirname "$0")/wasm.sh" --source-only
. "$(dirname "$0")/fuzz.sh" --source-only
