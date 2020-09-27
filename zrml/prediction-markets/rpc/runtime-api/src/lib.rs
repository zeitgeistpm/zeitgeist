//! Runtime API definition for the prediction markets pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sp_runtime::traits::{MaybeDisplay, MaybeFromStr};
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
    pub trait PredictionMarketsApi<MarketId, Hash> where
        MarketId: Codec,
        Hash: Codec,
    {
        fn market_outcome_share_id(market_id: MarketId, outcome: u16) -> Hash;
    }
}
