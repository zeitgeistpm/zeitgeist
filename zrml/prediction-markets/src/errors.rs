use frame_support::dispatch::DispatchError;

pub const NO_REPORT: DispatchError = DispatchError::Other("Report does not exist");
pub const NOT_RESOLVED: DispatchError = DispatchError::Other("Resolved outcome does not exist");
pub const OUTCOME_MISMATCH: DispatchError =
    DispatchError::Other("Submitted outcome does not match market type");
