mod additional_chain_spec;
mod battery_park;
mod dev;
mod local_testnet;

pub use additional_chain_spec::AdditionalChainSpec;
pub use battery_park::battery_park_config;
pub use dev::dev_config;
pub use local_testnet::local_testnet_config;
use sp_core::{Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use zeitgeist_primitives::types::{AccountId, Balance, Signature};
use zeitgeist_runtime::TokensConfig;

#[cfg(feature = "parachain")]
use {sp_runtime::Perbill, zeitgeist_primitives::ztg};

#[cfg(feature = "parachain")]
const DEFAULT_COLLATOR_INFLATION_INFO: parachain_staking::InflationInfo<Balance> =
    parachain_staking::InflationInfo {
        annual: parachain_staking::Range {
            ideal: Perbill::from_percent(ztg::STAKING),
            max: Perbill::from_percent(ztg::STAKING),
            min: Perbill::from_percent(ztg::STAKING),
        },
        expect: parachain_staking::Range {
            ideal: ztg::COLLATORS * zeitgeist_primitives::constants::BASE,
            min: ztg::COLLATORS * zeitgeist_primitives::constants::BASE,
            max: ztg::COLLATORS * zeitgeist_primitives::constants::BASE,
        },
        round: parachain_staking::Range {
            ideal: Perbill::from_parts(
                Perbill::from_perthousand(ztg::STAKING).deconstruct() / 8766,
            ),
            min: Perbill::from_parts(Perbill::from_perthousand(ztg::STAKING).deconstruct() / 8766),
            max: Perbill::from_parts(Perbill::from_perthousand(ztg::STAKING).deconstruct() / 8766),
        },
    };
const TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

#[cfg(feature = "parachain")]
pub type ChainSpec =
    sc_service::GenericChainSpec<zeitgeist_runtime::GenesisConfig, crate::chain_spec::Extensions>;
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
        #[cfg(feature = "parachain")]
        cf_reward: zeitgeist_runtime::CfRewardConfig {
            next_init: 0,
            conversion_rate: ksm_ztg_crowdloan_conversion_rate(),
        },
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
        pallet_vesting: zeitgeist_runtime::VestingConfig { vesting: vec![] },
        #[cfg(feature = "parachain")]
        parachain_info: zeitgeist_runtime::ParachainInfoConfig {
            parachain_id: acs.parachain_id,
        },
        #[cfg(feature = "parachain")]
        parachain_staking: zeitgeist_runtime::ParachainStakingConfig {
            inflation_config: acs.inflation_info,
            stakers: acs.stakers,
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
    pub para_id: u32,
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

#[cfg(feature = "parachain")]
fn ksm_ztg_crowdloan_conversion_rate() -> u128 {
    const DUMMY_TOTAL_KSM: u128 = 100_000;

    let crowdloan = Perbill::from_perthousand(ztg::CROWDLOAN);
    let parachain_lease = ztg::PARACHAIN_LEASE;
    let ztg_per_lease_period = crowdloan.saturating_reciprocal_mul(parachain_lease);
    ztg_per_lease_period
        .checked_div(DUMMY_TOTAL_KSM)
        .unwrap_or_default()
}
