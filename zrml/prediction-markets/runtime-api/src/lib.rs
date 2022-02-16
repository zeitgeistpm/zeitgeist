//! Runtime API definition for the prediction markets pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Codec, MaxEncodedLen};
use zeitgeist_primitives::types::Asset;

sp_api::decl_runtime_apis! {
    pub trait PredictionMarketsApi<MarketId, Hash> where
        MarketId: Codec + MaxEncodedLen,
        Hash: Codec,
    {
        fn market_outcome_share_id(market_id: MarketId, outcome: u16) -> Asset<MarketId>;
    }
}
