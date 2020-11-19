// use codec::FullCodec;
use sp_runtime::{
    // traits::{AtLeast32Bit, MaybeSerializeDeserialize},
    DispatchResult,
};
// use sp_std::fmt::Debug;

pub trait Swaps<Balance, Hash> {
    fn create_pool(assets: Vec<Hash>, swap_fee: Balance, weights: Vec<u128>) -> DispatchResult;
}
