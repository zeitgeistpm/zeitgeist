use crate::types::CategoryIndex;

/// The reported outcome of a market
#[derive(Clone, Debug, Eq, PartialEq, parity_scale_codec::Decode, parity_scale_codec::Encode)]
pub enum OutcomeReport {
    Categorical(CategoryIndex),
    Scalar(u128),
}
