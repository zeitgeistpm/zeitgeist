/// Parameters used by the `OwnedValues` storage.
///
/// # Types
///
/// * `BA`: BAlance
/// * `BN`: Block Number
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[derive(
    scale_info::TypeInfo,
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
pub struct OwnedValuesParams<BA, BN> {
    /// The number of blocks an account participated in a market period.
    pub participated_blocks: BN,
    /// Owned amount of perpetual incentives. Won't go away when accounts exist early and is not
    /// attached to any share
    pub perpetual_incentives: BA,
    /// Owned incentives. Related to the total number of shares.
    pub total_incentives: BA,
    /// Owned quantity of shares. Related to the total amount of incentives.
    pub total_shares: BA,
}
