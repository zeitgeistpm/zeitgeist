use alloc::vec::Vec;
use zeitgeist_primitives::types::PoolId;

#[derive(
    Clone,
    Debug,
    Default,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
)]
pub struct CommonPoolEventParams<AI> {
    pub pool_id: PoolId,
    pub who: AI,
}

#[derive(
    Clone,
    Debug,
    Default,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
)]
pub struct PoolAssetsEvent<AI, B> {
    pub bounds: Vec<B>,
    pub cpep: CommonPoolEventParams<AI>,
    pub transferred: Vec<B>,
}

#[derive(
    Clone,
    Debug,
    Default,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
)]
pub struct PoolAssetEvent<AI, B> {
    pub bound: B,
    pub cpep: CommonPoolEventParams<AI>,
    pub transferred: B,
}

#[derive(
    Clone,
    Debug,
    Default,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
)]
pub struct SwapEvent<AI, B> {
    pub asset_amount_in: B,
    pub asset_amount_out: B,
    pub asset_bound: B,
    pub cpep: CommonPoolEventParams<AI>,
    pub max_price: B,
}
