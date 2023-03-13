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

#[cfg(feature = "parachain")]
pub use crate::parachain_params::*;
pub use crate::parameters::*;
use alloc::vec;
use frame_support::{
    traits::{ConstU16, ConstU32, Contains, EitherOfDiverse, EqualPrivilegeOnly, InstanceFilter},
    weights::{constants::RocksDbWeight, ConstantMultiplier, IdentityFee},
};
use frame_system::EnsureRoot;
use pallet_collective::{EnsureProportionAtLeast, PrimeDefaultVote};
use pallet_transaction_payment::ChargeTransactionPayment;
use sp_runtime::traits::{AccountIdConversion, AccountIdLookup, BlakeTwo256};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use substrate_fixed::{types::extra::U33, FixedI128, FixedU128};
use zeitgeist_primitives::{constants::*, types::*};
use zrml_rikiddo::types::{EmaMarketVolume, FeeSigmoid, RikiddoSigmoidMV};
#[cfg(feature = "parachain")]
use {
    frame_support::traits::{AsEnsureOriginWithArg, Everything, Nothing},
    frame_system::EnsureSigned,
    xcm_builder::{EnsureXcmOrigin, FixedWeightBounds, LocationInverter},
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

pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("zeitgeist"),
    impl_name: create_runtime_str!("zeitgeist"),
    authoring_version: 1,
    spec_version: 44,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 20,
    state_version: 1,
};

#[derive(scale_info::TypeInfo)]
pub struct IsCallable;

// Currently disables Court, Rikiddo and creation of markets using Court or SimpleDisputes
// dispute mechanism.
impl Contains<Call> for IsCallable {
    fn contains(call: &Call) -> bool {
        #[cfg(feature = "parachain")]
        use cumulus_pallet_dmp_queue::Call::service_overweight;
        use frame_system::Call::{kill_prefix, set_code, set_code_without_checks};
        use pallet_collective::Call::set_members;
        use pallet_vesting::Call::force_vested_transfer;

        #[allow(clippy::match_like_matches_macro)]
        match call {
            // Membership is managed by the respective Membership instance
            Call::AdvisoryCommittee(set_members { .. }) => false,
            // See "balance.set_balance"
            Call::AssetManager(_) => false,
            Call::Balances(_) => false,
            // Membership is managed by the respective Membership instance
            Call::Council(set_members { .. }) => false,
            Call::Court(_) => false,
            #[cfg(feature = "parachain")]
            Call::DmpQueue(service_overweight { .. }) => false,
            Call::LiquidityMining(_) => false,
            #[cfg(feature = "parachain")]
            Call::ParachainStaking(_) => false,
            Call::PredictionMarkets(_) => false,
            // Call::PredictionMarkets(inner_call) => {
            //     match inner_call {
            //         // Disable Rikiddo markets
            //         create_market { scoring_rule: RikiddoSigmoidFeeMarketEma, .. } => false,
            //         edit_market { scoring_rule: RikiddoSigmoidFeeMarketEma, .. } => false,
            //         // Disable Court & SimpleDisputes dispute resolution mechanism
            //         create_market { dispute_mechanism: Court | SimpleDisputes, .. } => false,
            //         edit_market { dispute_mechanism: Court | SimpleDisputes, .. } => false,
            //         create_cpmm_market_and_deploy_assets {
            //             dispute_mechanism: Court | SimpleDisputes,
            //             ..
            //         } => false,
            //         _ => true,
            //     }
            // }
            Call::Styx(_) => false,
            Call::Swaps(_) => false,
            Call::System(inner_call) => {
                match inner_call {
                    // Some "waste" storage will never impact proper operation.
                    // Cleaning up storage should be done by pallets or independent migrations.
                    kill_prefix { .. } => false,
                    // See "killPrefix"

                    // kill_storage { .. } => false,

                    // A parachain uses ParachainSystem to enact and authorized a runtime upgrade.
                    // This ensure proper synchronization with the relay chain.
                    // Calling `setCode` will wreck the chain.
                    set_code { .. } => false,
                    // See "setCode"
                    set_code_without_checks { .. } => false,
                    // Setting the storage directly is a dangerous operation that can lead to an
                    // inconsistent state. There might be scenarios where this is helpful, however,
                    // a well reviewed migration is better suited for that.

                    // set_storage { .. } => false,
                    _ => true,
                }
            }
            // Membership is managed by the respective Membership instance
            Call::TechnicalCommittee(set_members { .. }) => false,
            // There should be no reason to force vested transfer.
            Call::Vesting(force_vested_transfer { .. }) => false,
            _ => true,
        }
    }
}

decl_common_types!();

#[cfg(feature = "with-global-disputes")]
create_runtime_with_additional_pallets!(
    GlobalDisputes: zrml_global_disputes::{Call, Event<T>, Pallet, Storage} = 59,
);

#[cfg(not(feature = "with-global-disputes"))]
create_runtime_with_additional_pallets!();

impl_config_traits!();
create_runtime_api!();
create_common_benchmark_logic!();
create_common_tests!();
