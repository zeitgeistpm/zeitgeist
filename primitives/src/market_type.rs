/// Defines the type of market.
/// All markets also have the `Invalid` resolution.
#[derive(
    Clone,
    Eq,
    PartialEq,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    sp_runtime::RuntimeDebug,
)]
pub enum MarketType {
    // A market with a number of categorical outcomes.
    Categorical(u16),
    // A market with a range of potential outcomes.
    Scalar(RangeInclusive<u128>),
}

// An inclusive range between the left side (lower) and right (upper).
type RangeInclusive<T> = (T, T);
