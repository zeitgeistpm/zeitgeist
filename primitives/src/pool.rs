use crate::types::{Asset, PoolStatus};
use alloc::{collections::BTreeMap, vec::Vec};

#[derive(
    Clone,
    Eq,
    PartialEq,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    sp_runtime::RuntimeDebug,
)]
pub struct Pool<Balance, MarketId> {
    pub assets: Vec<Asset<MarketId>>,
    pub market_id: MarketId,
    pub pool_status: PoolStatus,
    pub scoring_rule: ScoringRule,
    pub swap_fee: Balance,
    pub total_weight: Option<u128>,
    pub weights: Option<BTreeMap<Asset<MarketId>, u128>>,
}

impl<Balance, MarketId> Pool<Balance, MarketId>
where
    MarketId: Ord,
{
    pub fn bound(&self, asset: &Asset<MarketId>) -> bool {
        let weight = BTreeMap::get(&self.weights, asset);
        weight.is_some()
    }
}

#[derive(
    Clone,
    Eq,
    PartialEq,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    sp_runtime::RuntimeDebug,
)]
pub enum ScoringRule {
    CPMM,
    RikiddoSigmoidMarketEma,
}
