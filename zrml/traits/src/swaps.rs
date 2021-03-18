use sp_runtime::{DispatchError, FixedU128};
use sp_std::vec::Vec;

pub trait Swaps<AccountId, Hash> {
    fn do_create_pool(
        creator: AccountId,
        assets: Vec<Hash>,
        swap_fee: FixedU128,
        weights: Vec<FixedU128>,
    ) -> sp_std::result::Result<u128, DispatchError>;
}
