use crate::types::CategoryIndex;

/// The reported outcome of a market
#[derive(
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    parity_scale_codec::MaxEncodedLen,
    scale_info::TypeInfo,
    Clone,
    Debug,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum OutcomeReport {
    Categorical(CategoryIndex),
    Scalar(u128),
}
