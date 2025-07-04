// Copyright 2022-2025 Forecasting Technologies LTD.
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
    create_common_benchmark_logic, create_common_tests, create_genesis_config_preset,
    create_runtime, create_runtime_api, create_runtime_with_additional_pallets, decl_common_types,
    impl_config_traits,
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
use pallet_collective::{EnsureProportionAtLeast, PrimeDefaultVote};
use sp_runtime::traits::{AccountIdConversion, AccountIdLookup, BlakeTwo256};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use zeitgeist_primitives::types::*;
use zrml_swaps::Call::force_pool_exit;
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
    spec_version: 60,
    impl_version: 0,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 33,
    state_version: 1,
};

#[derive(scale_info::TypeInfo)]
pub struct IsCallable;

impl Contains<RuntimeCall> for IsCallable {
    fn contains(call: &RuntimeCall) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match call {
            RuntimeCall::Swaps(inner_call) => match inner_call {
                force_pool_exit { .. } => true,
                _ => false,
            },
            _ => true,
        }
    }
}

parameter_types! {
    pub RemovableMarketIds: Vec<u32> = vec![879u32, 877u32, 878u32, 880u32, 882u32];
}

decl_common_types!();

create_runtime_with_additional_pallets!(
    Sudo: pallet_sudo::{Call, Config<T>, Event<T>, Pallet, Storage} = 150,
);

impl pallet_sudo::Config for Runtime {
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

impl_config_traits!();
create_genesis_config_preset!(
    sudo: SudoConfig {
        key: Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
    },
);
create_runtime_api!();
create_common_benchmark_logic!();
create_common_tests!();
