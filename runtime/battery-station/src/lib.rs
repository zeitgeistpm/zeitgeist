// Copyright 2022-2023 Forecasting Technologies LTD.
// Copyright 2021-2022 Zeitgeist PM LLC.
//
// This file is part of Zeitgeist.
//
// Zeitgeist is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// Zeitgeist is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Zeitgeist. If not, see <https://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "512"]

extern crate alloc;

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use common_runtime::{
    create_common_benchmark_logic, create_common_tests, create_runtime, create_runtime_api,
    create_runtime_with_additional_pallets, decl_common_types, impl_config_traits,
};
pub use frame_system::{
    Call as SystemCall, CheckEra, CheckGenesis, CheckNonZeroSender, CheckNonce, CheckSpecVersion,
    CheckTxVersion, CheckWeight,
};
#[cfg(feature = "parachain")]
pub use pallet_author_slot_filter::EligibilityValue;
pub use pallet_balances::Call as BalancesCall;
use pallet_collective::EnsureProportionMoreThan;

#[cfg(feature = "parachain")]
pub use crate::parachain_params::*;
pub use crate::parameters::*;
use alloc::vec;
use frame_support::{
    traits::{ConstU32, Contains, EitherOfDiverse, EqualPrivilegeOnly, InstanceFilter},
    weights::{constants::RocksDbWeight, ConstantMultiplier, IdentityFee, Weight},
};
use frame_system::{EnsureRoot, EnsureWithSuccess};
use orml_currencies::Call::transfer;
use pallet_collective::{EnsureProportionAtLeast, PrimeDefaultVote};
use sp_runtime::{
    traits::{AccountIdConversion, AccountIdLookup, BlakeTwo256},
    DispatchError,
};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use substrate_fixed::{types::extra::U33, FixedI128, FixedU128};
use zeitgeist_primitives::{constants::*, types::*};
use zrml_prediction_markets::Call::{
    buy_complete_set, create_cpmm_market_and_deploy_assets, create_market,
    deploy_swap_pool_and_additional_liquidity, deploy_swap_pool_for_market, dispute, edit_market,
    redeem_shares, report, sell_complete_set,
};
use zrml_rikiddo::types::{EmaMarketVolume, FeeSigmoid, RikiddoSigmoidMV};
use zrml_swaps::Call::{
    pool_exit, pool_exit_with_exact_asset_amount, pool_exit_with_exact_pool_amount, pool_join,
    pool_join_with_exact_asset_amount, pool_join_with_exact_pool_amount, swap_exact_amount_in,
    swap_exact_amount_out,
};
#[cfg(feature = "parachain")]
use {
    frame_support::traits::{AsEnsureOriginWithArg, Everything, Nothing},
    xcm_builder::{EnsureXcmOrigin, FixedWeightBounds},
    xcm_config::{
        asset_registry::CustomAssetProcessor,
        config::{LocalOriginToLocation, XcmConfig, XcmOriginToTransactDispatchOrigin, XcmRouter},
    },
};

use frame_support::construct_runtime;

use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
    create_runtime_str,
    traits::Block as BlockT,
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult,
};

#[cfg(feature = "parachain")]
use nimbus_primitives::CanAuthor;
use sp_version::RuntimeVersion;

#[cfg(test)]
pub mod integration_tests;
#[cfg(feature = "parachain")]
pub mod parachain_params;
pub mod parameters;
#[cfg(feature = "parachain")]
pub mod xcm_config;

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("zeitgeist"),
    impl_name: create_runtime_str!("zeitgeist"),
    authoring_version: 1,
    spec_version: 48,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 23,
    state_version: 1,
};

#[derive(scale_info::TypeInfo)]
pub struct ContractsCallfilter;

impl Contains<RuntimeCall> for ContractsCallfilter {
    fn contains(runtime_call: &RuntimeCall) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match runtime_call {
            RuntimeCall::AssetManager(transfer { .. }) => true,
            RuntimeCall::PredictionMarkets(inner_call) => {
                match inner_call {
                    buy_complete_set { .. } => true,
                    deploy_swap_pool_and_additional_liquidity { .. } => true,
                    deploy_swap_pool_for_market { .. } => true,
                    dispute { .. } => true,
                    // Only allow CPMM markets using Authorized or SimpleDisputes dispute mechanism
                    create_market {
                        dispute_mechanism: MarketDisputeMechanism::Authorized,
                        scoring_rule: ScoringRule::CPMM,
                        ..
                    } => true,
                    create_cpmm_market_and_deploy_assets {
                        dispute_mechanism: MarketDisputeMechanism::Authorized,
                        ..
                    } => true,
                    edit_market {
                        dispute_mechanism: MarketDisputeMechanism::Authorized,
                        scoring_rule: ScoringRule::CPMM,
                        ..
                    } => true,
                    redeem_shares { .. } => true,
                    report { .. } => true,
                    sell_complete_set { .. } => true,
                    _ => false,
                }
            }
            RuntimeCall::Swaps(inner_call) => match inner_call {
                pool_exit { .. } => true,
                pool_exit_with_exact_asset_amount { .. } => true,
                pool_exit_with_exact_pool_amount { .. } => true,
                pool_join { .. } => true,
                pool_join_with_exact_asset_amount { .. } => true,
                pool_join_with_exact_pool_amount { .. } => true,
                swap_exact_amount_in { .. } => true,
                swap_exact_amount_out { .. } => true,
                _ => false,
            },
            _ => false,
        }
    }
}

#[derive(scale_info::TypeInfo)]
pub struct IsCallable;

// Currently disables Rikiddo.
impl Contains<RuntimeCall> for IsCallable {
    fn contains(call: &RuntimeCall) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match call {
            RuntimeCall::SimpleDisputes(_) => false,
            RuntimeCall::LiquidityMining(_) => false,
            RuntimeCall::PredictionMarkets(inner_call) => {
                match inner_call {
                    // Disable Rikiddo and SimpleDisputes markets
                    create_market {
                        scoring_rule: ScoringRule::RikiddoSigmoidFeeMarketEma, ..
                    } => false,
                    create_market {
                        dispute_mechanism: MarketDisputeMechanism::SimpleDisputes,
                        ..
                    } => false,
                    edit_market {
                        scoring_rule: ScoringRule::RikiddoSigmoidFeeMarketEma, ..
                    } => false,
                    create_cpmm_market_and_deploy_assets {
                        dispute_mechanism: MarketDisputeMechanism::SimpleDisputes,
                        ..
                    } => false,
                    edit_market {
                        dispute_mechanism: MarketDisputeMechanism::SimpleDisputes,
                        ..
                    } => false,
                    _ => true,
                }
            }
            _ => true,
        }
    }
}

decl_common_types!();

create_runtime_with_additional_pallets!(
    Sudo: pallet_sudo::{Call, Config<T>, Event<T>, Pallet, Storage} = 150,
);

impl pallet_sudo::Config for Runtime {
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
}

impl_config_traits!();
create_runtime_api!();
create_common_benchmark_logic!();
create_common_tests!();
