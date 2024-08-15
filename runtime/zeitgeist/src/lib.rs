// Copyright 2022-2024 Forecasting Technologies LTD.
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
#![recursion_limit = "1024"]

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
    traits::{ConstU32, Contains, EitherOfDiverse, EqualPrivilegeOnly, InstanceFilter, Nothing},
    weights::{constants::RocksDbWeight, ConstantMultiplier, IdentityFee, Weight},
};
use frame_system::{EnsureRoot, EnsureWithSuccess};
use pallet_collective::{EnsureProportionAtLeast, EnsureProportionMoreThan, PrimeDefaultVote};
use sp_runtime::traits::{AccountIdConversion, AccountIdLookup, BlakeTwo256};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use zeitgeist_primitives::types::*;
#[cfg(feature = "parachain")]
use {
    frame_support::traits::{AsEnsureOriginWithArg, Everything},
    xcm_builder::{EnsureXcmOrigin, FixedWeightBounds},
    xcm_config::{
        asset_registry::CustomAssetProcessor,
        config::{LocalOriginToLocation, XcmConfig, XcmOriginToTransactDispatchOrigin, XcmRouter},
    },
};

use frame_support::construct_runtime;

use sp_api::{impl_runtime_apis, BlockT};
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
    create_runtime_str,
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
    spec_version: 56,
    impl_version: 0,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 29,
    state_version: 1,
};

pub type ContractsCallfilter = Nothing;

#[derive(scale_info::TypeInfo)]
pub struct IsCallable;

impl Contains<RuntimeCall> for IsCallable {
    fn contains(runtime_call: &RuntimeCall) -> bool {
        #[cfg(feature = "parachain")]
        use cumulus_pallet_dmp_queue::Call::service_overweight;
        use frame_system::Call::{
            kill_prefix, kill_storage, set_code, set_code_without_checks, set_storage,
        };
        use orml_currencies::Call::update_balance;
        use pallet_balances::Call::{force_set_balance, force_transfer};
        use pallet_collective::Call::set_members;
        use pallet_contracts::Call::{
            call, call_old_weight, instantiate, instantiate_old_weight, remove_code,
            set_code as set_code_contracts,
        };
        use pallet_vesting::Call::force_vested_transfer;
        use zrml_prediction_markets::Call::{
            admin_move_market_to_closed, admin_move_market_to_resolved,
        };
        use zrml_swaps::Call::force_pool_exit;

        #[allow(clippy::match_like_matches_macro)]
        match runtime_call {
            // Membership is managed by the respective Membership instance
            RuntimeCall::AdvisoryCommittee(set_members { .. }) => false,
            // See "balance.force_set_balance"
            RuntimeCall::AssetManager(update_balance { .. }) => false,
            RuntimeCall::Balances(inner_call) => {
                match inner_call {
                    // Balances should not be set. All newly generated tokens be minted by well
                    // known and approved processes, like staking. However, this could be used
                    // in some cases to fund system accounts like the parachain sorveign account
                    // in case something goes terribly wrong (like a hack that draws the funds
                    // from such an account, see Maganta hack). Invoking this function one can
                    // also easily mess up consistency in regards to reserved tokens and locks.
                    force_set_balance { .. } => false,
                    // There should be no reason to force an account to transfer funds.
                    force_transfer { .. } => false,
                    _ => true,
                }
            }
            // Permissioned contracts: Only deployable via utility.dispatch_as(...)
            RuntimeCall::Contracts(inner_call) => match inner_call {
                call { .. } => true,
                call_old_weight { .. } => true,
                instantiate { .. } => true,
                instantiate_old_weight { .. } => true,
                remove_code { .. } => true,
                set_code_contracts { .. } => true,
                _ => false,
            },
            // Membership is managed by the respective Membership instance
            RuntimeCall::Council(set_members { .. }) => false,
            #[cfg(feature = "parachain")]
            RuntimeCall::DmpQueue(service_overweight { .. }) => false,
            RuntimeCall::PredictionMarkets(inner_call) => match inner_call {
                admin_move_market_to_closed { .. } => false,
                admin_move_market_to_resolved { .. } => false,
                _ => true,
            },
            RuntimeCall::Swaps(inner_call) => match inner_call {
                force_pool_exit { .. } => true,
                _ => false,
            },
            RuntimeCall::System(inner_call) => {
                match inner_call {
                    // Some "waste" storage will never impact proper operation.
                    // Cleaning up storage should be done by pallets or independent migrations.
                    kill_prefix { .. } => false,
                    // See "killPrefix"
                    kill_storage { .. } => false,
                    // A parachain uses ParachainSystem to enact and authorized a runtime upgrade.
                    // This ensure proper synchronization with the relay chain.
                    // Calling `setCode` will wreck the chain.
                    set_code { .. } => false,
                    // See "setCode"
                    set_code_without_checks { .. } => false,
                    // Setting the storage directly is a dangerous operation that can lead to an
                    // inconsistent state. There might be scenarios where this is helpful, however,
                    // a well reviewed migration is better suited for that.
                    set_storage { .. } => false,
                    _ => true,
                }
            }
            // Membership is managed by the respective Membership instance
            RuntimeCall::TechnicalCommittee(set_members { .. }) => false,
            // There should be no reason to force vested transfer.
            RuntimeCall::Vesting(force_vested_transfer { .. }) => false,
            _ => true,
        }
    }
}

parameter_types! {
    pub RemovableMarketIds: Vec<u32> = vec![];
}

decl_common_types!();

create_runtime_with_additional_pallets!();

impl_config_traits!();
create_runtime_api!();
create_common_benchmark_logic!();
create_common_tests!();
