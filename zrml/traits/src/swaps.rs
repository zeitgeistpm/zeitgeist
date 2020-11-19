use sp_runtime::DispatchResult;
use sp_std::vec::Vec;

pub trait Swaps<Balance, Hash> {
    fn create_pool(assets: Vec<Hash>, swap_fee: Balance, weights: Vec<u128>) -> DispatchResult;
}
