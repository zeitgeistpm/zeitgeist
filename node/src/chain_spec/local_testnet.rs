use crate::chain_spec::{
    generic_genesis, get_account_id_from_seed, AdditionalChainSpec, ChainSpec,
};
use sc_service::ChainType;
use sp_core::sr25519;

#[cfg(feature = "parachain")]
const STAKE_AMOUNT: u128 = 2_000 * zeitgeist_primitives::constants::BASE;

pub fn local_testnet_config(
    #[cfg(feature = "parachain")] parachain_id: cumulus_primitives_core::ParaId,
) -> Result<ChainSpec, String> {
    let wasm_binary = zeitgeist_runtime::WASM_BINARY
        .ok_or_else(|| "Development wasm binary not available".to_string())?;
    Ok(ChainSpec::from_genesis(
        "Local Testnet",
        "local_testnet",
        ChainType::Local,
        move || {
            generic_genesis(
                #[cfg(feature = "parachain")]
                AdditionalChainSpec {
                    inflation_info: crate::chain_spec::DEFAULT_COLLATOR_INFLATION_INFO,
                    stakers: vec![
                        (get_account_id_from_seed::<sr25519::Public>("Alice"), None, STAKE_AMOUNT),
                        (get_account_id_from_seed::<sr25519::Public>("Bob"), None, STAKE_AMOUNT),
                    ],
                    parachain_id,
                },
                #[cfg(not(feature = "parachain"))]
                AdditionalChainSpec {
                    initial_authorities: vec![
                        crate::chain_spec::authority_keys_from_seed("Alice"),
                        crate::chain_spec::authority_keys_from_seed("Bob"),
                    ],
                },
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                zeitgeist_primitives::types::Balance::MAX >> 4,
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                wasm_binary,
            )
        },
        vec![],
        None,
        None,
        None,
        #[cfg(feature = "parachain")]
        crate::chain_spec::Extensions {
            relay_chain: "rococo-local".into(),
            parachain_id: parachain_id.into(),
        },
        #[cfg(not(feature = "parachain"))]
        Default::default(),
    ))
}
