#!/usr/bin/env bash

EXTERNAL_WEIGHTS_PATH="./runtime/common/src/weights/"

# This script contains the configuration for other benchmarking scripts.
export FRAME_PALLETS=( frame_system pallet_balances pallet_bounties pallet_democracy \
                pallet_identity pallet_membership  pallet_multisig pallet_preimage \
                pallet_proxy pallet_scheduler pallet_timestamp pallet_treasury \
                pallet_utility pallet_vesting pallet_collective ) # pallet_grandpa )
export FRAME_PALLETS_RUNS="${FRAME_PALLETS_RUNS:-20}"
export FRAME_PALLETS_STEPS="${FRAME_PALLETS_STEPS:-50}"

export FRAME_WEIGHT_TEMPLATE="./misc/frame_weight_template.hbs"

export FRAME_PALLETS_PARACHAIN=( 
                cumulus_pallet_xcmp_queue \
                pallet_author_inherent \
                pallet_author_slot_filter \
                pallet_author_mapping \
                pallet_parachain_staking \
)
export FRAME_PALLETS_PARACHAIN_RUNS="${FRAME_PALLETS_PARACHAIN_RUNS:-$FRAME_PALLETS_RUNS}"
export FRAME_PALLETS_PARACHAIN_STEPS="${FRAME_PALLETS_PARACHAIN_STEPS:-$FRAME_PALLETS_STEPS}"

export ORML_PALLETS=( orml_currencies orml_tokens )
export ORML_PALLETS_RUNS="${ORML_PALLETS_RUNS:-20}"
export ORML_PALLETS_STEPS="${ORML_PALLETS_STEPS:-50}"
export ORML_WEIGHT_TEMPLATE="./misc/orml_weight_template.hbs"

export ZEITGEIST_PALLETS=( zrml_authorized zrml_court zrml_global_disputes zrml_liquidity_mining zrml_prediction_markets zrml_swaps zrml_styx )
export ZEITGEIST_PALLETS_RUNS="${ZEITGEIST_PALLETS_RUNS:-1000}"
export ZEITGEIST_PALLETS_STEPS="${ZEITGEIST_PALLETS_STEPS:-10}"
export ZEITGEIST_WEIGHT_TEMPLATE="./misc/weight_template.hbs"

export PROFILE="${PROFILE:-production}"
# this is used, because of profile `dev` with folder `debug`
if [ "$PROFILE" = "dev" ]; then
    export PROFILE_DIR="debug"
else
    export PROFILE_DIR="$PROFILE"
fi
export EXECUTION="${EXECUTION:-wasm}"
export ADDITIONAL="${ADDITIONAL:-}"
export ADDITIONAL_FEATURES="${ADDITIONAL_FEATURES:-with-global-disputes}"
