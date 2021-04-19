use crate::Asset;
use alloc::vec::Vec;
use frame_support::dispatch::DispatchError;

pub trait Swaps<AccountId> {
    type Balance;
    type Hash;
    type MarketId;

    fn create_pool(
        creator: AccountId,
        assets: Vec<Asset<Self::MarketId>>,
        swap_fee: Self::Balance,
        weights: Vec<u128>,
    ) -> Result<u128, DispatchError>;
}
