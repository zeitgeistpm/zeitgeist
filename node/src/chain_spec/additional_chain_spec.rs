#[cfg(feature = "parachain")]
use {
    cumulus_primitives_core::ParaId,
    parachain_staking::InflationInfo,
    zeitgeist_primitives::types::{AccountId, Balance},
};

#[cfg(feature = "parachain")]
pub struct AdditionalChainSpec {
    pub inflation_info: InflationInfo<Balance>,
    pub stakers: Vec<(AccountId, Option<AccountId>, Balance)>,
    pub parachain_id: ParaId,
}

#[cfg(not(feature = "parachain"))]
pub struct AdditionalChainSpec {
    pub initial_authorities: Vec<(
        sp_consensus_aura::sr25519::AuthorityId,
        sp_finality_grandpa::AuthorityId,
    )>,
}
