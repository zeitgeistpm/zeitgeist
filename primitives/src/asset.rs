use crate::types::SerdeWrapper;

/// The `Asset` enum represents all types of assets available in the Zeitgeist
/// system.
///
/// # Types
///
/// * `MI`: Market Id
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
pub enum Asset<MI> {
    CategoricalOutcome(MI, CategoryIndex),
    ScalarOutcome(MI, ScalarPosition),
    CombinatorialOutcome,
    PoolShare(SerdeWrapper<u128>),
    Ztg,
}

/// The index of the category for a `CategoricalOutcome` asset.
pub type CategoryIndex = u16;

/// In a scalar market, users can either choose a `Long` position,
/// meaning that they think the outcome will be closer to the upper bound
/// or a `Short` position meaning that they think the outcome will be closer
/// to the lower bound.
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
pub enum ScalarPosition {
    Long,
    Short,
}
