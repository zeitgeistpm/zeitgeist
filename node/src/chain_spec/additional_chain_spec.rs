#[cfg(feature = "parachain")]
use {
    cumulus_primitives_core::ParaId,
    nimbus_primitives::NimbusId,
    parachain_staking::InflationInfo,
    zeitgeist_primitives::types::{AccountId, Balance},
};

#[cfg(feature = "parachain")]
pub struct AdditionalChainSpec {
    pub candidates: Vec<(AccountId, NimbusId, Balance)>,
    pub inflation_info: InflationInfo<Balance>,
    pub nominations: Vec<(AccountId, AccountId, Balance)>,
    pub parachain_id: ParaId,
}

#[cfg(not(feature = "parachain"))]
pub struct AdditionalChainSpec {
    pub initial_authorities: Vec<(
        sp_consensus_aura::sr25519::AuthorityId,
        sp_finality_grandpa::AuthorityId,
    )>,
}
