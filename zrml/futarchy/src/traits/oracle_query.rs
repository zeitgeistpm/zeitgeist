use alloc::fmt::Debug;
use frame_support::pallet_prelude::Weight;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

pub trait OracleQuery:
    Clone + Debug + Decode + Encode + MaxEncodedLen + PartialEq + TypeInfo
{
    /// Evaluates the query at the current block and returns the weight consumed and a `bool`
    /// indicating whether the query evaluated positively.
    fn evaluate(&self) -> (Weight, bool);
}
