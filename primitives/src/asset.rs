/// # Types
///
/// * `H`: Share's hash
/// * `MI`: Market Id
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
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
pub enum Asset<H, MI> {
    Share(H),
    PredictionMarketShare(MI, u16),
    PoolShare(u128),
    Ztg,
}
