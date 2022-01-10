use crate::JurorStatus;

// Structure currently has only one field but acts as a container for possible future additions.
#[derive(scale_info::TypeInfo,Clone, Debug, PartialEq, parity_scale_codec::Decode, parity_scale_codec::Encode)]
pub struct Juror {
    pub(crate) status: JurorStatus,
}
