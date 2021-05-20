use crate::types::Asset;
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
    pub swap_fee: Balance,
    pub total_weight: u128,
    pub weights: BTreeMap<Asset<MarketId>, u128>,
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
