#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

extern crate alloc;

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use super::common::*;
pub use frame_system::{
    Call as SystemCall, CheckEra, CheckGenesis, CheckNonZeroSender, CheckNonce, CheckSpecVersion,
    CheckTxVersion, CheckWeight,
};
pub use pallet_transaction_payment::ChargeTransactionPayment;
#[cfg(feature = "parachain")]
pub use {pallet_author_slot_filter::EligibilityValue};

// Expose runtime
pub use {api, parameters::SS58Prefix, Call, Runtime, RuntimeApi, VERSION};
#[cfg(feature = "std")]
pub use {
    AdvisoryCommitteeMembershipConfig, BalancesConfig, CouncilMembershipConfig, GenesisConfig,
    LiquidityMiningConfig, SudoConfig, SystemConfig, TechnicalCommitteeMembershipConfig,
};
#[cfg(all(feature = "std", not(feature = "parachain")))]
pub use battery_station::{AuraConfig, GrandpaConfig};

// Expose functions and types required to construct node CLI
#[cfg(feature = "std")]
pub use common::native_version;
pub use common::{
    opaque::Block, ChargeTransactionPayment, CheckEra, CheckGenesis, CheckNonZeroSender,
    CheckNonce, CheckSpecVersion, CheckTxVersion, CheckWeight, SignedExtra, SignedPayload,
    SystemCall, UncheckedExtrinsic,
};

pub use frame_system::{
    Call as SystemCall, CheckEra, CheckGenesis, CheckNonZeroSender, CheckNonce, CheckSpecVersion,
    CheckTxVersion, CheckWeight,
};
#[cfg(feature = "parachain")]
pub use pallet_author_slot_filter::EligibilityValue;
pub use pallet_transaction_payment::ChargeTransactionPayment;

use alloc::vec;
use frame_support::{
    traits::{ConstU16, ConstU32, Contains, EnsureOneOf, EqualPrivilegeOnly, InstanceFilter},
    weights::{constants::RocksDbWeight, ConstantMultiplier, IdentityFee},
};
use frame_system::EnsureRoot;
use pallet_collective::{EnsureProportionAtLeast, PrimeDefaultVote};
#[cfg(not(feature = "parachain"))]
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
    generic,
    traits::{AccountIdConversion, AccountIdLookup, BlakeTwo256},
};
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


use frame_support::{construct_runtime, traits::Contains};

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
use zeitgeist_primitives::types::*;

#[cfg(feature = "parachain")]
mod xcm_config;
pub mod parachain_params;
pub mod parameters;

pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("zeitgeist"),
    impl_name: create_runtime_str!("zeitgeist"),
    authoring_version: 1,
    spec_version: 38,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 15,
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

create_runtime_with_additional_pallets!(
    // Others
    Sudo: pallet_sudo::{Call, Config<T>, Event<T>, Pallet, Storage} = 150,
);
create_runtime_apis!();

impl pallet_sudo::Config for Runtime {
    type Call = Call;
    type Event = Event;
}
