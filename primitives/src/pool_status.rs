/// The status of a pool. Closely related to the lifecycle of a market.
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
)]
pub enum PoolStatus {
    /// Shares can be normally negotiated.
    Active,
    /// No trading is allowed. The pool is waiting to be subsidized.
    CollectingSubsidy,
    /// No trading is allowed. Only liquidity awaiting redemption is present in the pool.
    Stale,
}
