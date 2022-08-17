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

//! RPC interface for the Swaps pallet.

use core::{fmt::Display, str::FromStr};
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use parity_scale_codec::Codec;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
    generic::BlockId,
    traits::{Block as BlockT, MaybeDisplay, MaybeFromStr, NumberFor},
};
use std::sync::Arc;
use zeitgeist_primitives::types::{Asset, SerdeWrapper};

pub use zrml_swaps_runtime_api::SwapsApi as SwapsRuntimeApi;

#[rpc]
pub trait SwapsApi<BlockHash, BlockNumber, PoolId, AccountId, Balance, MarketId>
where
    Balance: FromStr + Display + parity_scale_codec::MaxEncodedLen,
    MarketId: FromStr + Display + parity_scale_codec::MaxEncodedLen,
    PoolId: FromStr + Display,
{
    #[rpc(name = "swaps_poolSharesId")]
    fn pool_shares_id(
        &self,
        pool_id: PoolId,
        at: Option<BlockHash>,
    ) -> Result<Asset<SerdeWrapper<MarketId>>>;

    #[rpc(name = "swaps_poolAccountId")]
    fn pool_account_id(&self, pool_id: PoolId, at: Option<BlockHash>) -> Result<AccountId>;

    #[rpc(name = "swaps_getSpotPrice")]
    fn get_spot_price(
        &self,
        pool_id: PoolId,
        asset_in: Asset<MarketId>,
        asset_out: Asset<MarketId>,
        at: Option<BlockHash>,
    ) -> Result<SerdeWrapper<Balance>>;

    #[rpc(name = "swaps_getSpotPrices")]
    fn get_spot_prices(
        &self,
        pool_id: PoolId,
        asset_in: Asset<MarketId>,
        asset_out: Asset<MarketId>,
        blocks: Vec<BlockNumber>,
    ) -> Result<Vec<SerdeWrapper<Balance>>>;
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

impl From<Error> for i64 {
    fn from(e: Error) -> i64 {
        match e {
            Error::RuntimeError => 69,
        }
    }
}

macro_rules! get_spot_price_rslt {
    (
        $api_ref:expr,
        $asset_in:expr,
        $asset_out:expr,
        $at:expr,
        $pool_id:expr
    ) => {
        $api_ref.get_spot_price($at, $pool_id, $asset_in, $asset_out).map_err(|e| RpcError {
            code: ErrorCode::ServerError(Error::RuntimeError.into()),
            message: "Unable to get spot price.".into(),
            data: Some(format!("{:?}", e).into()),
        })
    };
}

impl<C, Block, PoolId, AccountId, Balance, MarketId>
    SwapsApi<<Block as BlockT>::Hash, NumberFor<Block>, PoolId, AccountId, Balance, MarketId>
    for Swaps<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: SwapsRuntimeApi<Block, PoolId, AccountId, Balance, MarketId>,
    PoolId: Clone + Codec + MaybeDisplay + MaybeFromStr,
    AccountId: Codec,
    Balance: Codec + MaybeDisplay + MaybeFromStr + parity_scale_codec::MaxEncodedLen,
    MarketId: Clone + Codec + MaybeDisplay + MaybeFromStr + parity_scale_codec::MaxEncodedLen,
{
    fn pool_shares_id(
        &self,
        pool_id: PoolId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Asset<SerdeWrapper<MarketId>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            //if the block hash is not supplied assume the best block
            self.client.info().best_hash));

        api.pool_shares_id(&at, pool_id).map_err(|e| RpcError {
            code: ErrorCode::ServerError(Error::RuntimeError.into()),
            message: "Unable to get pool shares identifier.".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn pool_account_id(
        &self,
        pool_id: PoolId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<AccountId> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            //if the block hash is not supplied assume the best block
            self.client.info().best_hash));

        api.pool_account_id(&at, pool_id).map_err(|e| RpcError {
            code: ErrorCode::ServerError(Error::RuntimeError.into()),
            message: "Unable to get pool account identifier.".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    /// If block hash is not supplied, the best block is assumed.
    fn get_spot_price(
        &self,
        pool_id: PoolId,
        asset_in: Asset<MarketId>,
        asset_out: Asset<MarketId>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<SerdeWrapper<Balance>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        get_spot_price_rslt!(api, asset_in, asset_out, &at, pool_id)
    }

    fn get_spot_prices(
        &self,
        pool_id: PoolId,
        asset_in: Asset<MarketId>,
        asset_out: Asset<MarketId>,
        blocks: Vec<NumberFor<Block>>,
    ) -> Result<Vec<SerdeWrapper<Balance>>> {
        let api = self.client.runtime_api();
        blocks
            .into_iter()
            .map(|block| {
                let hash = BlockId::number(block);
                get_spot_price_rslt!(
                    &api,
                    asset_in.clone(),
                    asset_out.clone(),
                    &hash,
                    pool_id.clone()
                )
            })
            .collect::<Result<Vec<_>>>()
    }
}
