//! Runtime API definition for the prediction markets pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_std::prelude::*;
use zeitgeist_primitives::Asset;

sp_api::decl_runtime_apis! {
    pub trait PredictionMarketsApi<MarketId, Hash> where
        MarketId: Codec,
        Hash: Codec,
    {
        fn market_outcome_share_id(market_id: MarketId, outcome: u16) -> Asset<Hash, MarketId>;
    }
}
