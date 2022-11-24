#!/usr/bin/env bash

# This script benchmarks every pallet that is used within Zeitgeist.
# Execute from the root of the project.

set -eou pipefail

# Configuration

if [ ! -d "./scripts/benchmarks" ]; then
    echo "Please execute this script from the root of the Zeitgeist project folder"
    exit 1
fi;

export FRAME_PALLETS_STEPS=1
export FRAME_PALLETS_RUNS=0

export ORML_PALLETS_STEPS=1
export ORML_PALLETS_RUNS=0

export ZEITGEIST_PALLETS_STEPS=1
export ZEITGEIST_PALLETS_RUNS=0

export PROFILE=release
export PROFILE_DIR=release
export ADDITIONAL=--detailed-log-output
export EXECUTION=native
# TODO(#848) Delete this, when global disputes is on main-net
export ADDITIONAL_FEATURES=with-global-disputes

source ./scripts/benchmarks/run_benchmarks.sh
