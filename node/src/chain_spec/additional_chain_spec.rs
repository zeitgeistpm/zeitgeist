#[cfg(feature = "parachain")]
use cumulus_primitives_core::ParaId;

#[cfg(feature = "parachain")]
pub struct AdditionalChainSpec {
    pub parachain_id: ParaId,
}

#[cfg(not(feature = "parachain"))]
pub struct AdditionalChainSpec {
    pub initial_authorities: Vec<(
        sp_consensus_aura::sr25519::AuthorityId,
        sp_finality_grandpa::AuthorityId,
    )>,
}
