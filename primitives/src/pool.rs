use crate::types::{Asset, PoolStatus};
use alloc::{collections::BTreeMap, vec::Vec};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(
    TypeInfo,
    Clone,
    Encode,
    Eq,
    Decode,
    PartialEq,
    RuntimeDebug,
)]
pub struct Pool<Balance, MarketId>
{
    pub assets: Vec<Asset<MarketId>>,
    pub base_asset: Option<Asset<MarketId>>,
    pub market_id: MarketId,
    pub pool_status: PoolStatus,
    pub scoring_rule: ScoringRule,
    pub swap_fee: Option<Balance>,
    pub total_subsidy: Option<Balance>,
    pub total_weight: Option<u128>,
    pub weights: Option<BTreeMap<Asset<MarketId>, u128>>,
}

impl<Balance, MarketId> Pool<Balance, MarketId>
where
    MarketId: Ord
{
    pub fn bound(&self, asset: &Asset<MarketId>) -> bool {
        if let Some(weights) = &self.weights {
            return BTreeMap::get(weights, asset).is_some();
        }

        false
    }
}

impl<Balance, MarketId> MaxEncodedLen for Pool<Balance, MarketId> where
    Balance: MaxEncodedLen,
    MarketId: MaxEncodedLen,
{
    fn max_encoded_len() -> usize {
        <Vec<Asset<MarketId>>>::max_encoded_len()
            .saturating_add(<Option<Asset<MarketId>>>::max_encoded_len())
            .saturating_add(MarketId::max_encoded_len())
            .saturating_add(PoolStatus::max_encoded_len())
            // We assume that at max. a 512 bit hash function is used
            .saturating_add(ScoringRule::max_encoded_len().saturating_mul(66))
            .saturating_add(<Option<Balance>>::max_encoded_len()).saturating_mul(2)
            .saturating_add(<Option<u128>>::max_encoded_len())
            .saturating_add(<Option<BTreeMap<Asset<MarketId>, u128>>>::max_encoded_len())
    }
}

#[derive(
    TypeInfo,
    Clone,
    Encode,
    Eq,
    Decode,
    MaxEncodedLen,
    PartialEq,
    RuntimeDebug,
)]
pub enum ScoringRule {
    CPMM,
    RikiddoSigmoidFeeMarketEma,
}
