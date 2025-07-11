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

mod additional_chain_spec;
#[cfg(feature = "with-battery-station-runtime")]
pub(crate) mod battery_station;
#[cfg(feature = "with-battery-station-runtime")]
mod dev;
#[cfg(feature = "with-zeitgeist-runtime")]
pub(crate) mod zeitgeist;

pub use additional_chain_spec::AdditionalChainSpec;
#[cfg(feature = "with-battery-station-runtime")]
pub use battery_station::battery_station_staging_config;
#[cfg(feature = "with-battery-station-runtime")]
pub use dev::dev_config;
use sc_service::Properties;
use sc_telemetry::TelemetryEndpoints;
#[cfg(feature = "with-zeitgeist-runtime")]
pub use zeitgeist::zeitgeist_staging_config;
use zeitgeist_primitives::{
    constants::BalanceFractionalDecimals,
    types::{AccountId, Balance},
};

cfg_if::cfg_if! {
    if #[cfg(feature = "parachain")] {
        // Common
        macro_rules! generate_inflation_config_function {
            ($runtime:ident) => {
                use sp_runtime::Perbill;

                pub(super) fn inflation_config(
                    annual_inflation_min: Perbill,
                    annual_inflation_ideal: Perbill,
                    annual_inflation_max: Perbill,
                    total_supply: zeitgeist_primitives::types::Balance
                ) -> pallet_parachain_staking::inflation::InflationInfo<zeitgeist_primitives::types::Balance> {
                    fn to_round_inflation(annual: pallet_parachain_staking::inflation::Range<Perbill>) -> pallet_parachain_staking::inflation::Range<Perbill> {
                        use pallet_parachain_staking::inflation::{
                            perbill_annual_to_perbill_round,
                        };
                        use $runtime::parachain_params::DefaultBlocksPerRound;

                        perbill_annual_to_perbill_round(
                            annual,
                            // rounds per year
                            u32::try_from(zeitgeist_primitives::constants::BLOCKS_PER_YEAR).unwrap() / DefaultBlocksPerRound::get()
                        )
                    }
                    let annual = pallet_parachain_staking::inflation::Range {
                        min: annual_inflation_min,
                        ideal: annual_inflation_ideal,
                        max: annual_inflation_max,
                    };
                    pallet_parachain_staking::inflation::InflationInfo {
                        // staking expectations
                        expect: pallet_parachain_staking::inflation::Range {
                            min: Perbill::from_percent(5).mul_floor(total_supply),
                            ideal: Perbill::from_percent(10).mul_floor(total_supply),
                            max: Perbill::from_percent(15).mul_floor(total_supply),
                        },
                        // annual inflation
                        annual,
                        round: to_round_inflation(annual),
                    }
                }
            }
        }

        pub(crate) use generate_inflation_config_function;
        pub type DummyChainSpec = sc_service::GenericChainSpec<Extensions>;
    } else {
        pub type DummyChainSpec = sc_service::GenericChainSpec;
    }
}

const POLKADOT_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const ZEITGEIST_TELEMETRY_URL: &str = "wss://telemetry.zeitgeist.pm/submit/";

#[derive(Clone)]
pub(crate) struct EndowedAccountWithBalance(AccountId, Balance);

macro_rules! generate_generic_genesis_function {
    ($runtime:ident, $($additional_genesis:tt)*) => {
        #[allow(dead_code)] // used in submodules
        pub(super) fn generic_genesis(
            acs: AdditionalChainSpec,
            endowed_accounts: Vec<EndowedAccountWithBalance>,
        ) -> $runtime::RuntimeGenesisConfig {
            $runtime::RuntimeGenesisConfig {
                // Common genesis
                advisory_committee: Default::default(),
                advisory_committee_membership: $runtime::AdvisoryCommitteeMembershipConfig {
                    members: vec![].try_into().unwrap(),
                    phantom: Default::default(),
                },
                #[cfg(feature = "parachain")]
                asset_registry: Default::default(),
                #[cfg(not(feature = "parachain"))]
                aura: $runtime::AuraConfig {
                    authorities: acs.initial_authorities.iter().map(|x| (x.0.clone())).collect(),
                },
                #[cfg(feature = "parachain")]
                author_filter: $runtime::AuthorFilterConfig {
                    eligible_count: EligibilityValue::new_unchecked(1),
                    ..Default::default()
                },
                #[cfg(feature = "parachain")]
                author_mapping: $runtime::AuthorMappingConfig {
                    mappings: acs
                        .candidates
                        .iter()
                        .cloned()
                        .map(|(account_id, author_id, _)| (author_id, account_id))
                        .collect(),
                },
                balances: $runtime::BalancesConfig {
                    balances: endowed_accounts.iter().cloned().map(|k| (k.0, k.1)).collect(),
                },
                council: Default::default(),
                council_membership: $runtime::CouncilMembershipConfig {
                    members: vec![].try_into().unwrap(),
                    phantom: Default::default(),
                },
                democracy: Default::default(),
                #[cfg(not(feature = "parachain"))]
                grandpa: $runtime::GrandpaConfig {
                    authorities: acs.initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
                    ..Default::default()
                },
                #[cfg(feature = "parachain")]
                parachain_info: $runtime::ParachainInfoConfig {
                    parachain_id: acs.parachain_id,
                    ..Default::default()
                },
                #[cfg(feature = "parachain")]
                parachain_staking: $runtime::ParachainStakingConfig {
                    blocks_per_round: acs.blocks_per_round,
                    candidates: acs
                        .candidates
                        .iter()
                        .cloned()
                        .map(|(account, _, bond)| (account, bond))
                        .collect(),
                    collator_commission: acs.collator_commission,
                    inflation_config: acs.inflation_info,
                    delegations: acs.nominations,
                    parachain_bond_reserve_percent: acs.parachain_bond_reserve_percent,
                    num_selected_candidates: acs.num_selected_candidates,
                },
                #[cfg(feature = "parachain")]
                parachain_system: Default::default(),
                #[cfg(feature = "parachain")]
                // Default should use the pallet configuration
                polkadot_xcm: PolkadotXcmConfig::default(),
                system: $runtime::SystemConfig::default(),
                technical_committee: Default::default(),
                technical_committee_membership: $runtime::TechnicalCommitteeMembershipConfig {
                    members: vec![].try_into().unwrap(),
                    phantom: Default::default(),
                },
                treasury: Default::default(),
                transaction_payment: Default::default(),
                tokens: Default::default(),
                vesting: Default::default(),

                // Additional genesis
                $($additional_genesis)*
            }
        }
    };
}

pub(crate) use generate_generic_genesis_function;

/// The extensions for the [`ChainSpec`].
#[cfg(feature = "parachain")]
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    sc_chain_spec::ChainSpecExtension,
)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
    /// The id of the Parachain.
    pub parachain_id: u32,
    /// The relay chain of the Parachain.
    pub relay_chain: String,
    /// Known bad block hashes.
    pub bad_blocks: sc_client_api::BadBlocks<polkadot_primitives::Block>,
}

#[cfg(feature = "parachain")]
impl Extensions {
    /// Try to get the extension from the given `ChainSpec`.
    pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
        sc_chain_spec::get_extension(chain_spec.extensions())
    }
}

fn token_properties(token_symbol: &str, ss58_prefix: u8) -> Properties {
    let mut properties = Properties::new();
    properties.insert("ss58Format".into(), ss58_prefix.into());
    properties.insert("tokenSymbol".into(), token_symbol.into());
    properties.insert("tokenDecimals".into(), BalanceFractionalDecimals::get().into());
    properties
}

fn telemetry_endpoints() -> Option<TelemetryEndpoints> {
    TelemetryEndpoints::new(vec![
        (POLKADOT_TELEMETRY_URL.into(), 0),
        (ZEITGEIST_TELEMETRY_URL.into(), 0),
    ])
    .ok()
}
