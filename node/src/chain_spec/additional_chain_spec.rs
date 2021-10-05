use cumulus_primitives_core::ParaId;
use nimbus_primitives::NimbusId;
use parachain_staking::InflationInfo;
use zeitgeist_primitives::types::{AccountId, Balance};

pub struct AdditionalChainSpec {
    pub candidates: Vec<(AccountId, NimbusId, Balance)>,
    pub crowdloan_fund_pot: Balance,
    pub inflation_info: InflationInfo<Balance>,
    pub nominations: Vec<(AccountId, AccountId, Balance)>,
    pub parachain_id: ParaId,
}
