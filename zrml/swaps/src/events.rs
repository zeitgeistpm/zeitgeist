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
pub struct PoolAssetsEvent<AI, AS, B> {
    pub assets: Vec<AS>,
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
pub struct PoolAssetEvent<AI, AS, B> {
    pub asset: AS,
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
pub struct SwapEvent<AI, AS, B> {
    pub asset_amount_in: B,
    pub asset_amount_out: B,
    pub asset_bound: B,
    pub asset_in: AS,
    pub asset_out: AS,
    pub cpep: CommonPoolEventParams<AI>,
    pub max_price: B,
}
