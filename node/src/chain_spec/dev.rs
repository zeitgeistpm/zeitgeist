use crate::chain_spec::{
    generic_genesis, get_account_id_from_seed, get_from_seed, token_properties, zeitgeist_wasm,
    AdditionalChainSpec, ChainSpec,
};
use sc_service::ChainType;
use sp_core::sr25519;
use zeitgeist_primitives::types::Balance;

const INITIAL_BALANCE: Balance = Balance::MAX >> 4;

pub fn dev_config(parachain_id: cumulus_primitives_core::ParaId) -> Result<ChainSpec, String> {
    let zeitgeist_wasm = zeitgeist_wasm()?;
    Ok(ChainSpec::from_genesis(
        "Development",
        "dev",
        ChainType::Local,
        move || {
            generic_genesis(
                AdditionalChainSpec {
                    candidates: vec![(
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_from_seed::<nimbus_primitives::NimbusId>("Alice"),
                        crate::chain_spec::DEFAULT_STAKING_AMOUNT,
                    )],
                    crowdloan_fund_pot: zeitgeist_primitives::constants::BASE.saturating_mul(100),
                    inflation_info: crate::chain_spec::DEFAULT_COLLATOR_INFLATION_INFO,
                    nominations: vec![],
                    parachain_id,
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
                INITIAL_BALANCE,
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                zeitgeist_wasm,
            )
        },
        vec![],
        None,
        None,
        Some(token_properties()),
        crate::chain_spec::Extensions {
            relay_chain: "rococo-dev".into(),
            parachain_id: parachain_id.into(),
        },
    ))
}
