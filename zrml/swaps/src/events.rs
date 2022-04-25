use alloc::vec::Vec;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use zeitgeist_primitives::types::PoolId;

#[derive(Clone, Debug, Decode, Default, Encode, Eq, Ord, PartialEq, PartialOrd, TypeInfo)]
pub struct CommonPoolEventParams<AI> {
    pub pool_id: PoolId,
    pub who: AI,
}

/// Common parameters of Balancer all-asset events.
#[derive(Clone, Debug, Decode, Default, Encode, Eq, Ord, PartialEq, PartialOrd, TypeInfo)]
pub struct PoolAssetsEvent<AI, AS, B> {
    pub assets: Vec<AS>,
    pub bounds: Vec<B>,
    pub cpep: CommonPoolEventParams<AI>,
    pub transferred: Vec<B>,
    pub pool_amount: B,
}

/// Common parameters of Balancer single-asset events.
#[derive(Clone, Debug, Decode, Default, Encode, Eq, Ord, PartialEq, PartialOrd, TypeInfo)]
pub struct PoolAssetEvent<AI, AS, B> {
    pub asset: AS,
    pub bound: B,
    pub cpep: CommonPoolEventParams<AI>,
    pub transferred: B,
    pub pool_amount: B,
}

#[derive(Clone, Debug, Decode, Default, Encode, Eq, Ord, PartialEq, PartialOrd, TypeInfo)]
pub struct SwapEvent<AI, AS, B> {
    pub asset_amount_in: B,
    pub asset_amount_out: B,
    pub asset_bound: B,
    pub asset_in: AS,
    pub asset_out: AS,
    pub cpep: CommonPoolEventParams<AI>,
    pub max_price: Option<B>,
}
