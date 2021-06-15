mod additional_chain_spec;
mod battery_park;
mod dev;
mod local_testnet;

pub use additional_chain_spec::AdditionalChainSpec;
pub use battery_park::battery_park_staging_config;
pub use dev::dev_config;
pub use local_testnet::local_testnet_config;
use sp_core::{Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use zeitgeist_primitives::types::{AccountId, Balance, Signature};
use zeitgeist_runtime::TokensConfig;

#[cfg(feature = "parachain")]
use {
    sp_runtime::{Perbill, Percent},
    zeitgeist_primitives::constants::{ztg, DefaultBlocksPerRound, MILLISECS_PER_BLOCK},
};

#[cfg(feature = "parachain")]
const DEFAULT_COLLATOR_INFLATION_INFO: parachain_staking::InflationInfo<Balance> = {
    let hours_per_year = 8766;
    let millisecs_per_year = hours_per_year * 60 * 60 * 1000;
    let round_millisecs = DefaultBlocksPerRound::get() as u64 * MILLISECS_PER_BLOCK;
    let rounds_per_year = millisecs_per_year / round_millisecs;

    let annual_inflation = Perbill::from_percent(ztg::STAKING);
    let expected_annual_amount = ztg::COLLATORS * zeitgeist_primitives::constants::BASE;
    let round_inflation_parts = annual_inflation.deconstruct() as u64 / rounds_per_year;
    let round_inflation = Perbill::from_parts(round_inflation_parts as _);

    parachain_staking::InflationInfo {
        annual: parachain_staking::Range {
            ideal: annual_inflation,
            max: annual_inflation,
            min: annual_inflation,
        },
        expect: parachain_staking::Range {
            ideal: expected_annual_amount,
            max: expected_annual_amount,
            min: expected_annual_amount,
        },
        round: parachain_staking::Range {
            ideal: round_inflation,
            min: round_inflation,
            max: round_inflation,
        },
    }
};
const POLKADOT_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const ZEITGEIST_TELEMETRY_URL: &str = "wss://telemetry.zeitgeist.pm/submit/";

#[cfg(feature = "parachain")]
pub type ChainSpec = sc_service::GenericChainSpec<zeitgeist_runtime::GenesisConfig, Extensions>;
#[cfg(not(feature = "parachain"))]
pub type ChainSpec = sc_service::GenericChainSpec<zeitgeist_runtime::GenesisConfig>;

type AccountPublic = <Signature as Verify>::Signer;

fn generic_genesis(
    acs: AdditionalChainSpec,
    endowed_accounts: Vec<AccountId>,
    initial_balance: Balance,
    root_key: AccountId,
    wasm_binary: &[u8],
) -> zeitgeist_runtime::GenesisConfig {
    zeitgeist_runtime::GenesisConfig {
        frame_system: zeitgeist_runtime::SystemConfig {
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        orml_tokens: TokensConfig::default(),
        #[cfg(not(feature = "parachain"))]
        pallet_aura: zeitgeist_runtime::AuraConfig {
            authorities: acs
                .initial_authorities
                .iter()
                .map(|x| (x.0.clone()))
                .collect(),
        },
        #[cfg(feature = "parachain")]
        pallet_author_mapping: zeitgeist_runtime::AuthorMappingConfig {
            mappings: acs
                .candidates
                .iter()
                .cloned()
                .map(|(account_id, author_id, _)| (author_id, account_id))
                .collect(),
        },
        #[cfg(feature = "parachain")]
        pallet_author_slot_filter: zeitgeist_runtime::AuthorFilterConfig {
            eligible_ratio: Percent::from_percent(50),
        },
        pallet_balances: zeitgeist_runtime::BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, initial_balance))
                .collect(),
        },
        #[cfg(not(feature = "parachain"))]
        pallet_grandpa: zeitgeist_runtime::GrandpaConfig {
            authorities: acs
                .initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        },
        pallet_sudo: zeitgeist_runtime::SudoConfig { key: root_key },
        #[cfg(feature = "parachain")]
        parachain_info: zeitgeist_runtime::ParachainInfoConfig {
            parachain_id: acs.parachain_id,
        },
        #[cfg(feature = "parachain")]
        parachain_staking: zeitgeist_runtime::ParachainStakingConfig {
            candidates: acs
                .candidates
                .iter()
                .cloned()
                .map(|(account, _, bond)| (account, bond))
                .collect(),
            inflation_config: acs.inflation_info,
            nominations: acs.nominations,
        },
    }
}

fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// The extensions for the [`ChainSpec`].
#[cfg(feature = "parachain")]
#[derive(
    Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sc_chain_spec::ChainSpecExtension,
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

#[cfg(not(feature = "parachain"))]
fn authority_keys_from_seed(
    s: &str,
) -> (
    sp_consensus_aura::sr25519::AuthorityId,
    sp_finality_grandpa::AuthorityId,
) {
    (
        get_from_seed::<sp_consensus_aura::sr25519::AuthorityId>(s),
        get_from_seed::<sp_finality_grandpa::AuthorityId>(s),
    )
}
