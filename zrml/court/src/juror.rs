use crate::JurorStatus;

// Structure currently has only one field but acts as a container for possible future additions.
#[derive(
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    parity_scale_codec::MaxEncodedLen,
    scale_info::TypeInfo,
    Clone,
    Debug,
    PartialEq,
    Eq,
)]
pub struct Juror {
    pub(crate) status: JurorStatus,
}
