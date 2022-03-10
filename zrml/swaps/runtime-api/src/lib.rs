//! Runtime API definition for the swaps pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Codec, MaxEncodedLen};
use sp_runtime::traits::{MaybeDisplay, MaybeFromStr};
use zeitgeist_primitives::types::{Asset, SerdeWrapper};

sp_api::decl_runtime_apis! {
    pub trait SwapsApi<PoolId, AccountId, Balance, MarketId> where
        PoolId: Codec,
        AccountId: Codec,
        Balance: Codec + MaybeDisplay + MaybeFromStr + MaxEncodedLen,
        MarketId: Codec + MaxEncodedLen,
    {
        fn pool_shares_id(pool_id: PoolId) -> Asset<SerdeWrapper<MarketId>>;
        fn pool_account_id(pool_id: PoolId) -> AccountId;
        fn get_spot_price(pool_id: PoolId, asset_in: Asset<MarketId>, asset_out: Asset<MarketId>) -> SerdeWrapper<Balance>;
    }
}
