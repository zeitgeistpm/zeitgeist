use crate::{BoundedCallOf, Config, OracleOf};
use frame_support::{CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound};
use frame_system::pallet_prelude::BlockNumberFor;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

// TODO Make config a generic, keeps things simple.
#[derive(
    CloneNoBound, Decode, Encode, Eq, MaxEncodedLen, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo,
)]
#[scale_info(skip_type_params(S, T))]
pub struct Proposal<T>
where
    T: Config,
{
    pub when: BlockNumberFor<T>,
    pub call: BoundedCallOf<T>,
    pub oracle: OracleOf<T>,
}
