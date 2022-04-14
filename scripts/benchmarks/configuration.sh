#!/usr/bin/env bash

# This script contains the configuration for other benchmarking scripts.

export FRAME_PALLETS=( frame_system pallet_balances pallet_collective pallet_democracy \
                pallet_identity pallet_membership  pallet_multisig pallet_preimage \
                pallet_scheduler pallet_timestamp pallet_treasury pallet_utility pallet_vesting \
                ) # pallet_grandpa )
export FRAME_PALLETS_RUNS=20
export FRAME_PALLETS_STEPS=50

# pallet_crowdloan_rewards benchmark lead to an error within the verify function (deprecated)
export FRAME_PALLETS_PARACHAIN=( pallet_author_slot_filter pallet_author_mapping \
                parachain_staking ) # pallet_crowdloan_rewards )
export FRAME_PALLETS_PARACHAIN_RUNS=$FRAME_PALLETS_RUNS
export FRAME_PALLETS_PARACHAIN_STEPS=$FRAME_PALLETS_STEPS

export ORML_PALLETS=( orml_currencies orml_tokens )
export ORML_PALLETS_RUNS=20
export ORML_PALLETS_STEPS=50

export ZEITGEIST_PALLETS=( zrml_authorized zrml_court zrml_liquidity_mining zrml_prediction_markets zrml_swaps )
export ZEITGEIST_PALLETS_RUNS=1000
export ZEITGEIST_PALLETS_STEPS=10