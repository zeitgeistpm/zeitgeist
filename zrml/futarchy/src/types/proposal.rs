use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use alloc::fmt::Debug;

// TODO Make config a generic, keeps things simple.
#[derive(Clone, Debug, Decode, Encode, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct Proposal<When, BoundedCall, OracleQuery>
where
    When: Clone + Debug + Decode + Encode + MaxEncodedLen + PartialEq + TypeInfo,
    BoundedCall: Clone + Debug + Decode + Encode + MaxEncodedLen + PartialEq + TypeInfo,
    OracleQuery: Clone + Debug + Decode + Encode + MaxEncodedLen + PartialEq + TypeInfo,
{
    pub when: When,
    pub call: BoundedCall,
    pub query: OracleQuery,
}
