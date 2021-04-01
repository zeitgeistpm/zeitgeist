//! RPC interface for the Swaps pallet.

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::U256;
use sp_runtime::{
    generic::BlockId,
    traits::{Block as BlockT, MaybeDisplay, MaybeFromStr},
};
use sp_std::convert::TryFrom;
use std::sync::Arc;
use zeitgeist_primitives::Asset;

pub use zrml_swaps_runtime_api::{BalanceInfo, SwapsApi as SwapsRuntimeApi};

#[rpc]
pub trait SwapsApi<BlockHash, PoolId, Hash, AccountId, Balance, BalanceType, MarketId>
where
    Balance: std::str::FromStr,
{
    #[rpc(name = "swaps_poolSharesId")]
    fn pool_shares_id(
        &self,
        pool_id: PoolId,
        at: Option<BlockHash>,
    ) -> Result<Asset<Hash, MarketId>>;

    #[rpc(name = "swaps_poolAccountId")]
    fn pool_account_id(&self, pool_id: PoolId, at: Option<BlockHash>) -> Result<AccountId>;

    #[rpc(name = "swaps_getSpotPrice")]
    fn get_spot_price(
        &self,
        pool_id: PoolId,
        asset_in: Asset<Hash, MarketId>,
        asset_out: Asset<Hash, MarketId>,
        at: Option<BlockHash>,
    ) -> Result<BalanceType>;
}

/// A struct that implements the [`SwapsApi`].
pub struct Swaps<C, B> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>,
}

impl<C, B> Swaps<C, B> {
    /// Create a new `PredictionMarkets` with the given reference to
    /// the client.
    pub fn new(client: Arc<C>) -> Self {
        Swaps {
            client,
            _marker: Default::default(),
        }
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

impl<C, Block, PoolId, Hash, AccountId, Balance, MarketId>
    SwapsApi<
        <Block as BlockT>::Hash,
        PoolId,
        Hash,
        AccountId,
        Balance,
        BalanceInfo<Balance>,
        MarketId,
    > for Swaps<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: SwapsRuntimeApi<Block, PoolId, Hash, AccountId, Balance, MarketId>,
    PoolId: Codec,
    Hash: Codec,
    AccountId: Codec,
    Balance: Codec + MaybeDisplay + MaybeFromStr + TryFrom<U256>,
    <Balance as TryFrom<U256>>::Error: sp_std::fmt::Debug,
    MarketId: Codec,
{
    fn pool_shares_id(
        &self,
        pool_id: PoolId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Asset<Hash, MarketId>> {
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

    fn get_spot_price(
        &self,
        pool_id: PoolId,
        asset_in: Asset<Hash, MarketId>,
        asset_out: Asset<Hash, MarketId>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<BalanceInfo<Balance>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            //if the block hash is not supplied assume the best block
            self.client.info().best_hash));

        api.get_spot_price(&at, pool_id, asset_in, asset_out)
            .map_err(|e| RpcError {
                code: ErrorCode::ServerError(Error::RuntimeError.into()),
                message: "Unable to get spot price.".into(),
                data: Some(format!("{:?}", e).into()),
            })
    }
}
