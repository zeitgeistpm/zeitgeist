use sp_runtime::DispatchError;
use sp_std::vec::Vec;

pub trait Swaps<AccountId, Balance, Hash> {
    fn do_create_pool(creator: AccountId, assets: Vec<Hash>, swap_fee: Balance, weights: Vec<u128>) -> sp_std::result::Result<u128, DispatchError>;
}
