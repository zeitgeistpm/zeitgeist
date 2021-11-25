mod additional_chain_spec;
#[cfg(not(feature = "parachain"))]
mod battery_park;
mod battery_station;
mod dev;
mod zeitgeist;

pub use additional_chain_spec::AdditionalChainSpec;
#[cfg(not(feature = "parachain"))]
pub use battery_park::battery_park_staging_config;
pub use battery_station::battery_station_staging_config;
pub use dev::dev_config;
use hex_literal::hex;
use jsonrpc_core::serde_json::{Map, Value};
use sc_telemetry::TelemetryEndpoints;
use sp_core::{crypto::UncheckedInto, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
pub use zeitgeist::zeitgeist_staging_config;
use zeitgeist_primitives::{
    constants::{
        ztg::{LIQUIDITY_MINING, LIQUIDITY_MINING_PTD},
        BASE,
        BalanceFractionalDecimals,
    },
    types::{AccountId, Balance, Signature},
};
use zeitgeist_runtime::SS58Prefix;
#[cfg(feature = "parachain")]
use {
    sp_runtime::{Perbill, Percent},
    zeitgeist_primitives::constants::{ztg, DefaultBlocksPerRound, MILLISECS_PER_BLOCK},
};

cfg_if::cfg_if! {
    if #[cfg(feature = "parachain")] {
        // Testnet
        const DEFAULT_STAKING_AMOUNT_TESTNET: u128 = 2_000 * BASE;
        const DEFAULT_INITIAL_CROWDLOAN_FUNDS_TESTNET: u128 = 100 * BASE;

        // Mainnet
        const DEFAULT_STAKING_AMOUNT_MAINNET: u128 = 64 * BASE;
        const DEFAULT_INITIAL_CROWDLOAN_FUNDS_MAINNET: u128 = 100 * BASE;

        // Common
        const DEFAULT_COLLATOR_INFLATION_INFO: parachain_staking::InflationInfo<Balance> = {
            let hours_per_year = 8766;
            let millisecs_per_year = hours_per_year * 60 * 60 * 1000;
            let round_millisecs = DefaultBlocksPerRound::get() as u64 * MILLISECS_PER_BLOCK as u64;
            let rounds_per_year = millisecs_per_year / round_millisecs;
        
            let annual_inflation = ztg::STAKING_PTD;
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

        pub type ChainSpec = sc_service::GenericChainSpec<zeitgeist_runtime::GenesisConfig, Extensions>;
    } else {
        pub type ChainSpec = sc_service::GenericChainSpec<zeitgeist_runtime::GenesisConfig>;
    }
}

const DEFAULT_INITIAL_BALANCE_TESTNET: u128 = 10_000 * BASE;
const DEFAULT_SUDO_BALANCE_MAINNET: u128 = 100 * BASE;
const POLKADOT_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const ZEITGEIST_TELEMETRY_URL: &str = "wss://telemetry.zeitgeist.pm/submit/";


type AccountPublic = <Signature as Verify>::Signer;
#[derive(Clone)]
struct EndowedAccountWithBalance(AccountId, Balance);

fn generic_genesis(
    acs: AdditionalChainSpec,
    endowed_accounts: Vec<EndowedAccountWithBalance>,
    root_key: AccountId,
    wasm_binary: &[u8],
) -> zeitgeist_runtime::GenesisConfig {
    zeitgeist_runtime::GenesisConfig {
        advisory_committee_collective: Default::default(),
        advisory_committee_membership: zeitgeist_runtime::AdvisoryCommitteeMembershipConfig {
            members: vec![],
            phantom: Default::default(),
        },
        #[cfg(not(feature = "parachain"))]
        aura: zeitgeist_runtime::AuraConfig {
            authorities: acs.initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        },
        #[cfg(feature = "parachain")]
        author_filter: zeitgeist_runtime::AuthorFilterConfig {
            eligible_ratio: Percent::from_percent(50),
        },
        #[cfg(feature = "parachain")]
        author_mapping: zeitgeist_runtime::AuthorMappingConfig {
            mappings: acs
                .candidates
                .iter()
                .cloned()
                .map(|(account_id, author_id, _)| (author_id, account_id))
                .collect(),
        },
        balances: zeitgeist_runtime::BalancesConfig {
            balances: endowed_accounts.iter().cloned().map(|k| (k.0, k.1)).collect(),
        },
        #[cfg(feature = "parachain")]
        crowdloan: zeitgeist_runtime::CrowdloanConfig { funded_amount: acs.crowdloan_fund_pot },
        #[cfg(not(feature = "parachain"))]
        grandpa: zeitgeist_runtime::GrandpaConfig {
            authorities: acs.initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
        },
        liquidity_mining: zeitgeist_runtime::LiquidityMiningConfig {
            initial_balance: LIQUIDITY_MINING,
            per_block_distribution: LIQUIDITY_MINING_PTD.mul_ceil(LIQUIDITY_MINING),
        },
        #[cfg(feature = "parachain")]
        parachain_info: zeitgeist_runtime::ParachainInfoConfig { parachain_id: acs.parachain_id },
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
        #[cfg(feature = "parachain")]
        parachain_system: Default::default(),
        sudo: zeitgeist_runtime::SudoConfig { key: root_key },
        system: zeitgeist_runtime::SystemConfig {
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        treasury: Default::default(),
        tokens: Default::default(),
        vesting: Default::default(),
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

#[cfg(feature = "parachain")]
fn additional_chain_spec_staging_testnet(
    parachain_id: cumulus_primitives_core::ParaId,
) -> AdditionalChainSpec {
    use zeitgeist_primitives::constants::BASE;

    AdditionalChainSpec {
        candidates: vec![(
            hex!["302f6d7467ae2d7e3b9b962bfc3b9d929da9fae5f1e8c977a031ddf721b0790d"].into(),
            hex!["e6ea0b63b2b5b7247a1e8280350a14c5f9e7745dec2fe3428b68aa4167d48e66"]
                .unchecked_into(),
            crate::chain_spec::DEFAULT_STAKING_AMOUNT_TESTNET,
        )],
        crowdloan_fund_pot: DEFAULT_INITIAL_CROWDLOAN_FUNDS_TESTNET,
        inflation_info: crate::chain_spec::DEFAULT_COLLATOR_INFLATION_INFO,
        nominations: vec![],
        parachain_id,
    }
}
#[cfg(not(feature = "parachain"))]
fn additional_chain_spec_staging_testnet() -> AdditionalChainSpec {
    AdditionalChainSpec {
        initial_authorities: vec![(
            // 5FCSJzvmeUW1hBo3ASnLzSxpUdn5QUDt1Eqobj1meiQB7mLu
            hex!["8a9a54bdf73fb4a757f5ab81fabe2f173922fdb92bb8b6e8bedf8b17fa38f500"]
                .unchecked_into(),
            // 5HGProUwcyCDMJDxjBBKbv8u7ehr5uoTBS3bckYHPcZMTifW
            hex!["e61786c6426b55a034f9c4b78dc57d4183927cef8e64b2e496225ed6fca41758"]
                .unchecked_into(),
        )],
    }
}

#[cfg(not(feature = "parachain"))]
fn authority_keys_from_seed(
    s: &str,
) -> (sp_consensus_aura::sr25519::AuthorityId, sp_finality_grandpa::AuthorityId) {
    (
        get_from_seed::<sp_consensus_aura::sr25519::AuthorityId>(s),
        get_from_seed::<sp_finality_grandpa::AuthorityId>(s),
    )
}

fn endowed_accounts_staging_testnet() -> Vec<EndowedAccountWithBalance> {
    vec![
        // 5D2L4ghyiYE8p2z7VNJo9JYwRuc8uzPWtMBqdVyvjRcsnw4P
        EndowedAccountWithBalance(
            hex!["2a6c61a907556e4c673880b5767dd4be08339ee7f2a58d5137d0c19ca9570a5c"].into(),
            DEFAULT_INITIAL_BALANCE_TESTNET
        ),
        // 5EeeZVU4SiPG6ZRY7o8aDcav2p2mZMdu3ZLzbREWuHktYdhX
        EndowedAccountWithBalance(
            hex!["725bb6fd13d52b3d6830e5a9faed1f6499ca0f5e8aa285df09490646e71e831b"].into(),
            DEFAULT_INITIAL_BALANCE_TESTNET
        ),
        // 5D9tF8w1FMSdz52bpiaQis1pCUZy5Gs6HcHS7gHxEzyq4XzU
        #[cfg(feature = "parachain")]
        EndowedAccountWithBalance(
            hex!["302f6d7467ae2d7e3b9b962bfc3b9d929da9fae5f1e8c977a031ddf721b0790d"].into(),
            DEFAULT_STAKING_AMOUNT_TESTNET
        ),
    ]
}

fn root_key_staging_testnet() -> AccountId {
    hex!["2a6c61a907556e4c673880b5767dd4be08339ee7f2a58d5137d0c19ca9570a5c"].into()
}

fn telemetry_endpoints() -> Option<TelemetryEndpoints> {
    TelemetryEndpoints::new(vec![
        (POLKADOT_TELEMETRY_URL.into(), 0),
        (ZEITGEIST_TELEMETRY_URL.into(), 0),
    ])
    .ok()
}

fn token_properties(token_symbol: &str) -> Map<String, Value> {
    let mut properties = Map::new();
    properties.insert("ss58Format".into(), SS58Prefix::get().into());
    properties.insert("tokenSymbol".into(), token_symbol.into());
    properties.insert("tokenDecimals".into(), BalanceFractionalDecimals::get().into());
    properties
}

fn zeitgeist_wasm() -> Result<&'static [u8], String> {
    zeitgeist_runtime::WASM_BINARY.ok_or_else(|| "WASM binary is not available".to_string())
}
