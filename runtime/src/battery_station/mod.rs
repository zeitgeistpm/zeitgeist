#![cfg(feature = "runtime-battery-station")]
#[macro_use]

use frame_support::{construct_runtime, traits::Contains};
use super::common::{create_runtime, create_runtime_with_additional_pallets};

pub mod parachain_params;
pub mod parameters;

create_runtime_with_additional_pallets!(
    // Others
    Sudo: pallet_sudo::{Call, Config<T>, Event<T>, Pallet, Storage} = 150,
);

impl pallet_sudo::Config for Runtime {
    type Call = Call;
    type Event = Event;
}

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
                    | Call::Sudo(_) => true,

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
                    | Call::Sudo(_) => true,

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