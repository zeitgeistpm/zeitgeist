use crate::Error;
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

    pub fn get_weight_rslt<T>(&self, asset: &Asset<MarketId>) -> Result<&u128, Error<T>> {
        self.weights.get(asset).ok_or(Error::<T>::AssetNotBound)
    }
}
