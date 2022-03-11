use parity_scale_codec::{Decode, Encode, MaxEncodedLen};

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
    Decode,
    Default,
    Encode,
    Eq,
    MaxEncodedLen,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub struct OwnedValuesParams<BA, BN>
where
    BA: MaxEncodedLen,
    BN: MaxEncodedLen,
{
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
