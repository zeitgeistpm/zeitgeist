use crate::types::{CategoryIndex, PoolId, SerdeWrapper};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

/// The `Asset` enum represents all types of assets available in the Zeitgeist
/// system.
///
/// # Types
///
/// * `MI`: Market Id
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[derive(
    Clone, Copy, Debug, Decode, Eq, Encode, MaxEncodedLen, Ord, PartialEq, PartialOrd, TypeInfo,
)]
pub enum Asset<MI: MaxEncodedLen> {
    CategoricalOutcome(MI, CategoryIndex),
    ScalarOutcome(MI, ScalarPosition),
    CombinatorialOutcome,
    PoolShare(SerdeWrapper<PoolId>),
    Ztg,
}

/// In a scalar market, users can either choose a `Long` position,
/// meaning that they think the outcome will be closer to the upper bound
/// or a `Short` position meaning that they think the outcome will be closer
/// to the lower bound.
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[derive(
    Clone, Copy, Debug, Decode, Eq, Encode, MaxEncodedLen, Ord, PartialEq, PartialOrd, TypeInfo,
)]
pub enum ScalarPosition {
    Long,
    Short,
}
