#![cfg(feature = "runtime-zeitgeist")]

use super::common::*;
pub use frame_system::{
    Call as SystemCall, CheckEra, CheckGenesis, CheckNonZeroSender, CheckNonce, CheckSpecVersion,
    CheckTxVersion, CheckWeight,
};
pub use pallet_transaction_payment::ChargeTransactionPayment;
pub use parameters::*;
#[cfg(feature = "parachain")]
pub use {pallet_author_slot_filter::EligibilityValue, parachain_params::*};

use frame_support::{
    construct_runtime,
    traits::{ConstU16, ConstU32, Contains, EnsureOneOf, EqualPrivilegeOnly, InstanceFilter},
    weights::{constants::RocksDbWeight, ConstantMultiplier, IdentityFee},
};
use frame_system::EnsureRoot;
use pallet_collective::{EnsureProportionAtLeast, PrimeDefaultVote};
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
    create_runtime_str, generic,
    traits::{AccountIdConversion, AccountIdLookup, BlakeTwo256, Block as BlockT},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult,
};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use zeitgeist_primitives::{constants::*, types::*};
#[cfg(feature = "parachain")]
use {
    frame_support::traits::{Everything, Nothing},
    frame_system::EnsureSigned,
    nimbus_primitives::{CanAuthor, NimbusId},
    xcm_builder::{EnsureXcmOrigin, FixedWeightBounds, LocationInverter},
    xcm_config::XcmConfig,
};

pub mod parachain_params;
pub mod parameters;

create_runtime_with_additional_pallets!();
create_runtime_apis!();

#[derive(scale_info::TypeInfo)]
pub struct IsCallable;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "parachain", feature = "txfilter"))] {
        // Restricted parachain.
        impl Contains<Call> for IsCallable {
            fn contains(call: &Call) -> bool {
                match call {
                    // Allowed calls:
                    Call::AdvisoryCommittee(_)
                    | Call::AdvisoryCommitteeMembership(_)
                    | Call::AuthorInherent(_)
                    | Call::AuthorFilter(_)
                    | Call::AuthorMapping(_)
                    | Call::Balances(_)
                    | Call::Council(_)
                    | Call::CouncilMembership(_)
                    | Call::Crowdloan(_)
                    | Call::AssetManager(_)
                    | Call::Democracy(_)
                    | Call::DmpQueue(_)
                    | Call::Identity(_)
                    | Call::MultiSig(_)
                    | Call::ParachainStaking(_)
                    | Call::ParachainSystem(_)
                    | Call::PolkadotXcm(_)
                    | Call::Preimage(_)
                    | Call::Proxy(_)
                    | Call::Scheduler(_)
                    | Call::System(_)
                    | Call::TechnicalCommittee(_)
                    | Call::TechnicalCommitteeMembership(_)
                    | Call::Timestamp(_)
                    | Call::Treasury(_)
                    | Call::Utility(_)
                    | Call::Vesting(_)
                    | Call::XcmpQueue(_) => true,

                    // Prohibited calls:
                    Call::Authorized(_)
                    | Call::Court(_)
                    | Call::LiquidityMining(_)
                    | Call::Swaps(_)
                    | Call::PredictionMarkets(_) => false,
                }
            }
        }
    // Restricted standalone chain.
    } else if #[cfg(all(feature = "txfilter", not(feature = "parachain")))] {
        impl Contains<Call> for IsCallable {
            fn contains(call: &Call) -> bool {
                match call {
                    // Allowed calls:
                    Call::AdvisoryCommittee(_)
                    | Call::AdvisoryCommitteeMembership(_)
                    | Call::Balances(_)
                    | Call::Council(_)
                    | Call::CouncilMembership(_)
                    | Call::AssetManager(_)
                    | Call::Democracy(_)
                    | Call::Grandpa(_)
                    | Call::Identity(_)
                    | Call::MultiSig(_)
                    | Call::Preimage(_)
                    | Call::Proxy(_)
                    | Call::Scheduler(_)
                    | Call::System(_)
                    | Call::TechnicalCommittee(_)
                    | Call::TechnicalCommitteeMembership(_)
                    | Call::Timestamp(_)
                    | Call::Treasury(_)
                    | Call::Utility(_)
                    | Call::Vesting(_) => true,

                    // Prohibited calls:
                    Call::Authorized(_)
                    | Call::Court(_)
                    | Call::LiquidityMining(_)
                    | Call::Swaps(_)
                    | Call::PredictionMarkets(_)=> false,
                }
            }
        }
    // Unrestricted (no "txfilter" feature) chains.
    // Currently disables Rikiddo and markets using Court or SimpleDisputes dispute mechanism.
    // Will be relaxed for testnet once runtimes are separated.
    } else {
        impl Contains<Call> for IsCallable {
            fn contains(call: &Call) -> bool {
                use zrml_prediction_markets::Call::{create_market, create_cpmm_market_and_deploy_assets};
                use zeitgeist_primitives::types::{ScoringRule::RikiddoSigmoidFeeMarketEma, MarketDisputeMechanism::{Court, SimpleDisputes}};

                match call {
                    Call::PredictionMarkets(inner_call) => {
                        match inner_call {
                            // Disable Rikiddo markets
                            create_market { scoring_rule: RikiddoSigmoidFeeMarketEma, .. } => false,
                            // Disable Court & SimpleDisputes dispute resolution mechanism
                            create_market { dispute_mechanism: Court | SimpleDisputes, .. } => false,
                            create_cpmm_market_and_deploy_assets { dispute_mechanism: Court | SimpleDisputes, .. } => false,
                            _ => true
                        }
                    }
                    Call::LiquidityMining(_) => false,
                    _ => true
                }
            }
        }
    }
}

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