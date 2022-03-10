#[derive(
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    parity_scale_codec::MaxEncodedLen,
    scale_info::TypeInfo,
    Clone,
    Debug,
    PartialEq,
)]
pub enum JurorStatus {
    Ok,
    Tardy,
}
