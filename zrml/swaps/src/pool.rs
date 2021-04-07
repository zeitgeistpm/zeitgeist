use alloc::{collections::BTreeMap, vec::Vec};
use zeitgeist_primitives::Asset;

#[derive(
    Clone,
    Eq,
    PartialEq,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    sp_runtime::RuntimeDebug,
)]
pub struct Pool<Balance, Hash, MarketId> {
    pub assets: Vec<Asset<Hash, MarketId>>,
    pub swap_fee: Balance,
    pub total_weight: u128,
    pub weights: BTreeMap<Asset<Hash, MarketId>, u128>,
}

impl<Balance, Hash, MarketId> Pool<Balance, Hash, MarketId>
where
    Hash: Ord,
    MarketId: Ord,
{
    pub fn bound(&self, asset: &Asset<Hash, MarketId>) -> bool {
        let weight = BTreeMap::get(&self.weights, asset);
        weight.is_some()
    }
}
