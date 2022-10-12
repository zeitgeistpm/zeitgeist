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
#![recursion_limit = "256"]

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

#[cfg(feature = "parachain")]
pub use crate::parachain_params::*;
pub use crate::parameters::*;
use alloc::vec;
use frame_support::{
    traits::{ConstU16, ConstU32, Contains, EnsureOneOf, EqualPrivilegeOnly, InstanceFilter},
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
    frame_support::traits::{Everything, Nothing},
    frame_system::EnsureSigned,
    xcm_builder::{EnsureXcmOrigin, FixedWeightBounds, LocationInverter},
    xcm_config::XcmConfig,
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
use nimbus_primitives::{CanAuthor, NimbusId};
use sp_version::RuntimeVersion;

#[cfg(feature = "parachain")]
pub mod parachain_params;
pub mod parameters;
#[cfg(feature = "parachain")]
pub mod xcm_config;

pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("zeitgeist"),
    impl_name: create_runtime_str!("zeitgeist"),
    authoring_version: 1,
    spec_version: 40,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 17,
    state_version: 1,
};

#[derive(scale_info::TypeInfo)]
pub struct IsCallable;

// Currently disables Court, Rikiddo and creation of markets using Court or SimpleDisputes
// dispute mechanism.
impl Contains<Call> for IsCallable {
    fn contains(call: &Call) -> bool {
        use zeitgeist_primitives::types::{
            MarketDisputeMechanism::{Court, SimpleDisputes},
            ScoringRule::RikiddoSigmoidFeeMarketEma,
        };
        use zrml_prediction_markets::Call::{create_cpmm_market_and_deploy_assets, create_market};

        #[allow(clippy::match_like_matches_macro)]
        match call {
            Call::Court(_) => false,
            Call::LiquidityMining(_) => false,
            Call::PredictionMarkets(inner_call) => {
                match inner_call {
                    // Disable Rikiddo markets
                    create_market { scoring_rule: RikiddoSigmoidFeeMarketEma, .. } => false,
                    // Disable Court & SimpleDisputes dispute resolution mechanism
                    create_market { dispute_mechanism: Court | SimpleDisputes, .. } => false,
                    create_cpmm_market_and_deploy_assets {
                        dispute_mechanism: Court | SimpleDisputes,
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
    // Others
    Sudo: pallet_sudo::{Call, Config<T>, Event<T>, Pallet, Storage} = 150,
);

impl pallet_sudo::Config for Runtime {
    type Call = Call;
    type Event = Event;
}

impl_config_traits!();
create_runtime_api!();
create_common_benchmark_logic!();
create_common_tests!();
