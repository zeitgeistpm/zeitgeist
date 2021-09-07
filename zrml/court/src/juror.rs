use crate::JurorStatus;

/// * Types
///
/// * `B`: Balance
#[derive(Debug, PartialEq, parity_scale_codec::Decode, parity_scale_codec::Encode)]
pub struct Juror {
    pub(crate) status: JurorStatus,
}
