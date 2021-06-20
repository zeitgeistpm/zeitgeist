use crate::chain_spec::{
    generic_genesis, telemetry_endpoints, token_properties, zeitgeist_wasm, AdditionalChainSpec,
    ChainSpec,
};
use hex_literal::hex;
use sc_service::ChainType;
use sp_core::crypto::UncheckedInto;
use zeitgeist_primitives::constants::BASE;

pub fn battery_park_staging_config(
    #[cfg(feature = "parachain")] parachain_id: cumulus_primitives_core::ParaId,
) -> Result<ChainSpec, String> {
    let zeitgeist_wasm = zeitgeist_wasm()?;

    Ok(ChainSpec::from_genesis(
        "Zeitgeist - Battery Park Staging",
        "battery_park_staging",
        ChainType::Live,
        move || {
            generic_genesis(
                #[cfg(feature = "parachain")]
                AdditionalChainSpec {
                    candidates: vec![(
                        hex!["7225b9e36cc33da17d7f97053c7110559f21967afc09867345367b7de241d03c"]
                            .into(),
                        hex!["e6ea0b63b2b5b7247a1e8280350a14c5f9e7745dec2fe3428b68aa4167d48e66"]
                            .unchecked_into(),
                        crate::chain_spec::DEFAULT_STAKING_AMOUNT,
                    )],
                    inflation_info: crate::chain_spec::DEFAULT_COLLATOR_INFLATION_INFO,
                    nominations: vec![],
                    parachain_id,
                },
                #[cfg(not(feature = "parachain"))]
                AdditionalChainSpec {
                    initial_authorities: vec![(
                        // 5FCSJzvmeUW1hBo3ASnLzSxpUdn5QUDt1Eqobj1meiQB7mLu
                        hex!["8a9a54bdf73fb4a757f5ab81fabe2f173922fdb92bb8b6e8bedf8b17fa38f500"]
                            .unchecked_into(),
                        // 5HGProUwcyCDMJDxjBBKbv8u7ehr5uoTBS3bckYHPcZMTifW
                        hex!["e61786c6426b55a034f9c4b78dc57d4183927cef8e64b2e496225ed6fca41758"]
                            .unchecked_into(),
                    )],
                },
                vec![
                    // 5D2L4ghyiYE8p2z7VNJo9JYwRuc8uzPWtMBqdVyvjRcsnw4P
                    hex!["2a6c61a907556e4c673880b5767dd4be08339ee7f2a58d5137d0c19ca9570a5c"].into(),
                    // 5EeeZVU4SiPG6ZRY7o8aDcav2p2mZMdu3ZLzbREWuHktYdhX
                    hex!["725bb6fd13d52b3d6830e5a9faed1f6499ca0f5e8aa285df09490646e71e831b"].into(),
                    // 5EeNXHgaiWZAwZuZdDndJYcRTKuGHkXM2bdGE6LqWCw1bHW7
                    hex!["7225b9e36cc33da17d7f97053c7110559f21967afc09867345367b7de241d03c"].into(),
                ],
                10_000 * BASE,
                hex!["2a6c61a907556e4c673880b5767dd4be08339ee7f2a58d5137d0c19ca9570a5c"].into(),
                zeitgeist_wasm,
            )
        },
        vec![],
        telemetry_endpoints(),
        Some("battery_park_staging"),
        Some(token_properties()),
        #[cfg(feature = "parachain")]
        crate::chain_spec::Extensions {
            relay_chain: "rococo".into(),
            parachain_id: parachain_id.into(),
        },
        #[cfg(not(feature = "parachain"))]
        Default::default(),
    ))
}
