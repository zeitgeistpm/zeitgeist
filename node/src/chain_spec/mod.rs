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
#[cfg(feature = "with-raumgeist-runtime")]
pub(crate) mod raumgeist;
#[cfg(feature = "with-zeitgeist-runtime")]
pub(crate) mod zeitgeist;

pub use additional_chain_spec::AdditionalChainSpec;
#[cfg(feature = "with-battery-station-runtime")]
pub use battery_station::battery_station_staging_config;
#[cfg(feature = "with-battery-station-runtime")]
pub use dev::dev_config;
use jsonrpc_core::serde_json::{Map, Value};
#[cfg(feature = "with-raumgeist-runtime")]
pub use raumgeist::raumgeist_staging_config;
use sc_telemetry::TelemetryEndpoints;
#[cfg(feature = "with-battery-station-runtime")]
use sp_core::{Pair, Public};
#[cfg(feature = "with-battery-station-runtime")]
use sp_runtime::traits::{IdentifyAccount, Verify};
#[cfg(feature = "with-zeitgeist-runtime")]
pub use zeitgeist::zeitgeist_staging_config;
#[cfg(feature = "with-battery-station-runtime")]
use zeitgeist_primitives::types::Signature;
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
                    annual_inflation: Perbill,
                    total_supply: zeitgeist_primitives::types::Balance
                ) -> parachain_staking::inflation::InflationInfo<zeitgeist_primitives::types::Balance> {
                    fn to_round_inflation(annual: parachain_staking::inflation::Range<Perbill>) -> parachain_staking::inflation::Range<Perbill> {
                        use parachain_staking::inflation::{
                            perbill_annual_to_perbill_round,
                        };
                        use $runtime::parachain_params::DefaultBlocksPerRound;

                        perbill_annual_to_perbill_round(
                            annual,
                            // rounds per year
                            u32::try_from(zeitgeist_primitives::constants::BLOCKS_PER_YEAR).unwrap() / DefaultBlocksPerRound::get()
                        )
                    }
                    let annual = parachain_staking::inflation::Range {
                        min: annual_inflation,
                        ideal: annual_inflation,
                        max: annual_inflation,
                    };
                    let total_max = annual_inflation.mul_floor(total_supply);
                    parachain_staking::inflation::InflationInfo {
                        // staking expectations
                        expect: parachain_staking::inflation::Range {
                            min: Perbill::from_percent(20).mul_floor(total_max),
                            ideal: Perbill::from_percent(50).mul_floor(total_max),
                            max: total_max
                        },
                        // annual inflation
                        annual,
                        round: to_round_inflation(annual),
                    }
                }
            }
        }

        pub(crate) use generate_inflation_config_function;
        pub type DummyChainSpec = sc_service::GenericChainSpec<(), Extensions>;
    } else {
        pub type DummyChainSpec = sc_service::GenericChainSpec<()>;
    }
}

const POLKADOT_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const ZEITGEIST_TELEMETRY_URL: &str = "wss://telemetry.zeitgeist.pm/submit/";

#[derive(Clone)]
pub(crate) struct EndowedAccountWithBalance(AccountId, Balance);

macro_rules! generate_generic_genesis_function {
    ($runtime:ident, $($additional_genesis:tt)*) => {
        pub(super) fn generic_genesis(
            acs: AdditionalChainSpec,
            endowed_accounts: Vec<EndowedAccountWithBalance>,
            wasm_binary: &[u8],
        ) -> $runtime::GenesisConfig {
            $runtime::GenesisConfig {
                // Common genesis
                advisory_committee: Default::default(),
                advisory_committee_membership: $runtime::AdvisoryCommitteeMembershipConfig {
                    members: vec![],
                    phantom: Default::default(),
                },
                #[cfg(not(feature = "parachain"))]
                aura: $runtime::AuraConfig {
                    authorities: acs.initial_authorities.iter().map(|x| (x.0.clone())).collect(),
                },
                #[cfg(feature = "parachain")]
                author_filter: $runtime::AuthorFilterConfig {
                    eligible_count: EligibilityValue::new_unchecked(1),
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
                    members: vec![],
                    phantom: Default::default(),
                },
                #[cfg(feature = "parachain")]
                crowdloan: $runtime::CrowdloanConfig { funded_amount: acs.crowdloan_fund_pot },
                democracy: Default::default(),
                #[cfg(not(feature = "parachain"))]
                grandpa: $runtime::GrandpaConfig {
                    authorities: acs.initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
                },
                liquidity_mining: $runtime::LiquidityMiningConfig {
                    initial_balance: LIQUIDITY_MINING,
                    per_block_distribution: LIQUIDITY_MINING_PTD.mul_ceil(LIQUIDITY_MINING),
                },
                #[cfg(feature = "parachain")]
                parachain_info: $runtime::ParachainInfoConfig { parachain_id: acs.parachain_id },
                #[cfg(feature = "parachain")]
                parachain_staking: $runtime::ParachainStakingConfig {
                    candidates: acs
                        .candidates
                        .iter()
                        .cloned()
                        .map(|(account, _, bond)| (account, bond))
                        .collect(),
                    inflation_config: acs.inflation_info,
                    delegations: acs.nominations,
                },
                #[cfg(feature = "parachain")]
                parachain_system: Default::default(),
                #[cfg(feature = "parachain")]
                // Default should use the pallet configuration
                polkadot_xcm: PolkadotXcmConfig::default(),
                system: $runtime::SystemConfig { code: wasm_binary.to_vec() },
                technical_committee: Default::default(),
                technical_committee_membership: $runtime::TechnicalCommitteeMembershipConfig {
                    members: vec![],
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

#[cfg(feature = "with-battery-station-runtime")]
type AccountPublic = <Signature as Verify>::Signer;
#[cfg(feature = "with-battery-station-runtime")]
fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

#[cfg(feature = "with-battery-station-runtime")]
fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

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
}

#[cfg(feature = "parachain")]
impl Extensions {
    /// Try to get the extension from the given `ChainSpec`.
    pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
        sc_chain_spec::get_extension(chain_spec.extensions())
    }
}

fn token_properties(token_symbol: &str, ss58_prefix: u8) -> Map<String, Value> {
    let mut properties = Map::new();
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
