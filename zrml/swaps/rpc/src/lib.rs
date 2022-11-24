// Copyright 2021-2022 Zeitgeist PM LLC.
//
// This file is part of Zeitgeist.
//
// Zeitgeist is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// Zeitgeist is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Zeitgeist. If not, see <https://www.gnu.org/licenses/>.

#![doc = include_str!("../README.md")]

extern crate alloc;

use alloc::sync::Arc;
use core::{fmt::Display, str::FromStr};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
use parity_scale_codec::{Codec, MaxEncodedLen};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
    generic::BlockId,
    traits::{Block as BlockT, MaybeDisplay, MaybeFromStr, NumberFor},
};
use zeitgeist_primitives::types::{Asset, SerdeWrapper};

pub use zrml_swaps_runtime_api::SwapsApi as SwapsRuntimeApi;

#[rpc(client, server)]
pub trait SwapsApi<BlockHash, BlockNumber, PoolId, AccountId, Balance, MarketId>
where
    Balance: FromStr + Display + parity_scale_codec::MaxEncodedLen,
    MarketId: FromStr + Display + parity_scale_codec::MaxEncodedLen,
    PoolId: FromStr + Display,
{
    #[method(name = "swaps_poolSharesId", aliases = ["swaps_poolSharesIdAt"])]
    async fn pool_shares_id(
        &self,
        pool_id: PoolId,
        at: Option<BlockHash>,
    ) -> RpcResult<Asset<SerdeWrapper<MarketId>>>;

    #[method(name = "swaps_poolAccountId", aliases = ["swaps_poolAccountIdAt"])]
    async fn pool_account_id(&self, pool_id: PoolId, at: Option<BlockHash>)
    -> RpcResult<AccountId>;

    #[method(name = "swaps_getSpotPrice", aliases = ["swaps_getSpotPriceAt"])]
    async fn get_spot_price(
        &self,
        pool_id: PoolId,
        asset_in: Asset<MarketId>,
        asset_out: Asset<MarketId>,
        at: Option<BlockHash>,
        with_fees: bool,
    ) -> RpcResult<SerdeWrapper<Balance>>;

    #[method(name = "swaps_getSpotPrices")]
    async fn get_spot_prices(
        &self,
        pool_id: PoolId,
        asset_in: Asset<MarketId>,
        asset_out: Asset<MarketId>,
        blocks: Vec<BlockNumber>,
        with_fees: bool,
    ) -> RpcResult<Vec<SerdeWrapper<Balance>>>;
}

/// A struct that implements the [`SwapsApi`].
pub struct Swaps<C, B> {
    client: Arc<C>,
    _marker: core::marker::PhantomData<B>,
}

impl<C, B> Swaps<C, B> {
    /// Create a new `PredictionMarkets` with the given reference to
    /// the client.
    pub fn new(client: Arc<C>) -> Self {
        Swaps { client, _marker: Default::default() }
    }
}

pub enum Error {
    /// The call to the runtime failed.
    RuntimeError,
}

impl From<Error> for i32 {
    fn from(e: Error) -> i32 {
        match e {
            Error::RuntimeError => 1,
        }
    }
}

#[async_trait]
impl<C, Block, PoolId, AccountId, Balance, MarketId>
    SwapsApiServer<<Block as BlockT>::Hash, NumberFor<Block>, PoolId, AccountId, Balance, MarketId>
    for Swaps<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: SwapsRuntimeApi<Block, PoolId, AccountId, Balance, MarketId>,
    PoolId: Clone + Codec + MaybeDisplay + MaybeFromStr + Send + 'static,
    AccountId: Clone + Display + Codec + Send + 'static,
    Balance: Codec + MaybeDisplay + MaybeFromStr + MaxEncodedLen + Send + 'static,
    MarketId: Clone + Codec + MaybeDisplay + MaybeFromStr + MaxEncodedLen + Send + 'static,
{
    async fn pool_shares_id(
        &self,
        pool_id: PoolId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Asset<SerdeWrapper<MarketId>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            //if the block hash is not supplied assume the best block
            self.client.info().best_hash));

        let res = api.pool_shares_id(&at, pool_id).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to get pool shares identifier.",
                Some(e.to_string()),
            ))
        })?;
        Ok(res)
    }

    async fn pool_account_id(
        &self,
        pool_id: PoolId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<AccountId> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            //if the block hash is not supplied assume the best block
            self.client.info().best_hash));

        let res = api.pool_account_id(&at, &pool_id).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to get pool account identifier.",
                Some(e.to_string()),
            ))
        })?;
        Ok(res)
    }

    /// If block hash is not supplied, the best block is assumed.
    async fn get_spot_price(
        &self,
        pool_id: PoolId,
        asset_in: Asset<MarketId>,
        asset_out: Asset<MarketId>,
        at: Option<<Block as BlockT>::Hash>,
        with_fees: bool,
    ) -> RpcResult<SerdeWrapper<Balance>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        let res =
            api.get_spot_price(&at, &pool_id, &asset_in, &asset_out, with_fees).map_err(|e| {
                CallError::Custom(ErrorObject::owned(
                    Error::RuntimeError.into(),
                    "Unable to get spot price.",
                    Some(e.to_string()),
                ))
            })?;
        Ok(res)
    }

    async fn get_spot_prices(
        &self,
        pool_id: PoolId,
        asset_in: Asset<MarketId>,
        asset_out: Asset<MarketId>,
        blocks: Vec<NumberFor<Block>>,
        with_fees: bool,
    ) -> RpcResult<Vec<SerdeWrapper<Balance>>> {
        let api = self.client.runtime_api();
        blocks
            .into_iter()
            .map(|block| {
                let hash = BlockId::number(block);
                let res = api
                    .get_spot_price(&hash, &pool_id, &asset_in, &asset_out, with_fees)
                    .map_err(|e| {
                        CallError::Custom(ErrorObject::owned(
                            Error::RuntimeError.into(),
                            "Unable to get spot price.",
                            Some(e.to_string()),
                        ))
                    })?;
                Ok(res)
            })
            .collect()
    }
}
