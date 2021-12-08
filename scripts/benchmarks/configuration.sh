#!/usr/bin/env bash

# This script contains the configuration for other benchmarking scripts.

export FRAME_PALLETS=( frame_system pallet_balances pallet_collective pallet_identity pallet_membership \
                pallet_timestamp pallet_treasury pallet_utility pallet_vesting ) # pallet_grandpa )

# pallet_crowdloan_rewards benchmark lead to an error within the verify function (deprecated)
export FRAME_PALLETS_PARACHAIN=( pallet_author_mapping parachain_staking ) # pallet_crowdloan_rewards )
export ORML_PALLETS=( orml_currencies orml_tokens )
export ZEITGEIST_PALLETS=( zrml_authorized zrml_court zrml_liquidity_mining zrml_prediction_markets zrml_swaps )