//! Runtime API definition for the swaps pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
    pub trait SwapsApi<PoolId, Hash, AccountId, Balance> where
        PoolId: Codec,
        Hash: Codec,
        AccountId: Codec,
        Balance: Codec,
    {
        fn pool_shares_id(pool_id: PoolId) -> Hash;
        fn pool_account_id(pool_id: PoolId) -> AccountId;
        fn get_spot_price(pool_id: PoolId, asset_in: Hash, asset_out: Hash) -> Balance;
    }
}
