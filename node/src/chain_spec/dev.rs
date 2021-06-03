#[cfg(feature = "parachain")]
use crate::chain_spec::get_from_seed;
use crate::chain_spec::{
    generic_genesis, get_account_id_from_seed, AdditionalChainSpec, ChainSpec,
};
use sc_service::ChainType;
use sp_core::sr25519;

#[cfg(feature = "parachain")]
const STAKE_AMOUNT: u128 = 2_000 * zeitgeist_primitives::constants::BASE;

pub fn dev_config(
    #[cfg(feature = "parachain")] parachain_id: cumulus_primitives_core::ParaId,
) -> Result<ChainSpec, String> {
    let wasm_binary = zeitgeist_runtime::WASM_BINARY
        .ok_or_else(|| "Development wasm binary not available".to_string())?;
    Ok(ChainSpec::from_genesis(
        "Development",
        "dev",
        ChainType::Local,
        move || {
            generic_genesis(
                #[cfg(feature = "parachain")]
                AdditionalChainSpec {
                    candidates: vec![
                        (
                            get_account_id_from_seed::<sr25519::Public>("Alice"),
                            get_from_seed::<nimbus_primitives::NimbusId>("Alice"),
                            STAKE_AMOUNT,
                        ),
                        (
                            get_account_id_from_seed::<sr25519::Public>("Bob"),
                            get_from_seed::<nimbus_primitives::NimbusId>("Bob"),
                            STAKE_AMOUNT,
                        ),
                    ],
                    inflation_info: crate::chain_spec::DEFAULT_COLLATOR_INFLATION_INFO,
                    nominations: vec![],
                    parachain_id,
                },
                #[cfg(not(feature = "parachain"))]
                AdditionalChainSpec {
                    initial_authorities: vec![crate::chain_spec::authority_keys_from_seed("Alice")],
                },
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
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
