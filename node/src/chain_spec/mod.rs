mod battery_park;
mod dev;
mod local_testnet;

pub use battery_park::battery_park_staging_config;
pub use dev::dev_config;
pub use local_testnet::local_testnet_config;
use sp_core::{Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use zeitgeist_primitives::types::{AccountId, Signature};
use zeitgeist_runtime::{BalancesConfig, TokensConfig};

const TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

#[cfg(feature = "parachain")]
pub type ChainSpec =
    sc_service::GenericChainSpec<zeitgeist_runtime::GenesisConfig, crate::chain_spec::Extensions>;
#[cfg(not(feature = "parachain"))]
pub type ChainSpec = sc_service::GenericChainSpec<zeitgeist_runtime::GenesisConfig>;

type AccountPublic = <Signature as Verify>::Signer;

fn generic_genesis(
    endowed_accounts: Vec<AccountId>,
    #[cfg(feature = "parachain")] id: cumulus_primitives_core::ParaId,
    #[cfg(not(feature = "parachain"))] initial_authorities: Vec<(
        sp_consensus_aura::sr25519::AuthorityId,
        sp_finality_grandpa::AuthorityId,
    )>,
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
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        },
        pallet_balances: BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
        },
        #[cfg(not(feature = "parachain"))]
        pallet_grandpa: zeitgeist_runtime::GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        },
        pallet_sudo: zeitgeist_runtime::SudoConfig { key: root_key },
        #[cfg(feature = "parachain")]
        parachain_info: zeitgeist_runtime::ParachainInfoConfig { parachain_id: id },
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
