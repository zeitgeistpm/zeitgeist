#[derive(Debug, PartialEq, parity_scale_codec::Decode, parity_scale_codec::Encode)]
pub enum JurorStatus {
    Ok,
    Tardy,
}
