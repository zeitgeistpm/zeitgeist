/// Parameters used by the `OwnedValues` storage.
///
/// # Types
///
/// * `BAL`: BALance
/// * `BNR`: Block NumbeR
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
pub struct OwnedValuesParams<BAL, BNR> {
    /// The number of blocks an account participated in a market period.
    pub participated_blocks: BNR,
    /// Owned amount of perpetual incentives. Won't go away when accounts exist early and is not
    /// attached to any share
    pub perpetual_incentives: BAL,
    /// Owned incentives. Related to the total number of shares.
    pub total_incentives: BAL,
    /// Owned quantity of shares. Related to the total amount of incentives.
    pub total_shares: BAL,
}
