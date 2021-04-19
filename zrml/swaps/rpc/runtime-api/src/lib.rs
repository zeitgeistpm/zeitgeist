//! Runtime API definition for the swaps pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Codec, Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sp_runtime::traits::{MaybeDisplay, MaybeFromStr};
use zeitgeist_primitives::Asset;

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct BalanceInfo<Balance> {
    #[cfg_attr(
        feature = "std",
        serde(bound(serialize = "Balance: std::fmt::Display"))
    )]
    #[cfg_attr(feature = "std", serde(serialize_with = "serialize_as_string"))]
    #[cfg_attr(
        feature = "std",
        serde(bound(deserialize = "Balance: std::str::FromStr"))
    )]
    #[cfg_attr(feature = "std", serde(deserialize_with = "deserialize_from_string"))]
    pub amount: Balance,
}

#[cfg(feature = "std")]
fn serialize_as_string<S: Serializer, T: std::fmt::Display>(
    t: &T,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&t.to_string())
}

#[cfg(feature = "std")]
fn deserialize_from_string<'de, D: Deserializer<'de>, T: std::str::FromStr>(
    deserializer: D,
) -> Result<T, D::Error> {
    let s = String::deserialize(deserializer)?;
    s.parse::<T>()
        .map_err(|_| serde::de::Error::custom("Parse from string failed"))
}

sp_api::decl_runtime_apis! {
    pub trait SwapsApi<PoolId, AccountId, Balance, MarketId> where
        PoolId: Codec,
        AccountId: Codec,
        Balance: Codec + MaybeDisplay + MaybeFromStr,
        MarketId: Codec
    {
        fn pool_shares_id(pool_id: PoolId) -> Asset<MarketId>;
        fn pool_account_id(pool_id: PoolId) -> AccountId;
        fn get_spot_price(pool_id: PoolId, asset_in: Asset<MarketId>, asset_out: Asset<MarketId>) -> BalanceInfo<Balance>;
    }
}
