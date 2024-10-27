use crate::MarketIdOf;
use alloc::fmt::Debug;
use core::iter;
use frame_support::{CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{traits::Get, BoundedVec};

#[derive(
    CloneNoBound, Decode, Encode, Eq, MaxEncodedLen, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo,
)]
#[scale_info(skip_type_params(MaxMarkets))]
pub(crate) enum PoolType<MarketId, MaxMarkets>
where
    MarketId: Clone + Decode + Debug + Encode + MaxEncodedLen + PartialEq + Eq + TypeInfo,
    MaxMarkets: Get<u32>,
{
    Standard(MarketId),
    Combinatorial(BoundedVec<MarketId, MaxMarkets>),
}

impl<MarketId, MaxMarkets> PoolType<MarketId, MaxMarkets>
where
    MarketId: Clone + Decode + Debug + Encode + MaxEncodedLen + PartialEq + Eq + TypeInfo,
    MaxMarkets: Get<u32>,
{
    pub fn iter(&self) -> Box<dyn Iterator<Item = &MarketId> + '_> {
        match self {
            PoolType::Standard(market_id) => Box::new(iter::once(market_id)),
            PoolType::Combinatorial(market_ids) => Box::new(market_ids.iter()),
        }
    }
}
