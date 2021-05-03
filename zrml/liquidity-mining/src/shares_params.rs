/// Parameters used by `SoldShares`.
///
/// # Types
///
/// * `B`: Balance
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
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
pub struct SharesParams<B> {
    /// Shares in a single block
    pub shares: B,
    /// The sum of all owned shares
    pub total_shares: B,
}
