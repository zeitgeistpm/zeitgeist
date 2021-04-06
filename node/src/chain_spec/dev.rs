use crate::chain_spec::{generic_genesis, get_account_id_from_seed, ChainSpec};
use sc_service::ChainType;
use sp_core::sr25519;

pub fn dev_config(
    #[cfg(feature = "parachain")] id: cumulus_primitives_core::ParaId,
) -> Result<ChainSpec, String> {
    let wasm_binary = zeitgeist_runtime::WASM_BINARY
        .ok_or("Development wasm binary not available".to_string())?;
    Ok(ChainSpec::from_genesis(
        "Development",
        "dev",
        ChainType::Local,
        move || {
            generic_genesis(
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                ],
                #[cfg(feature = "parachain")]
                id,
                #[cfg(not(feature = "parachain"))]
                vec![crate::chain_spec::authority_keys_from_seed("Alice")],
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
            relay_chain: "rococo-dev".into(),
            para_id: id.into(),
        },
        #[cfg(not(feature = "parachain"))]
        Default::default(),
    ))
}
