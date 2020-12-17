//! RPC interface for the Swaps pallet.

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
    generic::BlockId,
    traits::Block as BlockT,
};
use std::sync::Arc;

pub use self::gen_client::Client as PmClient;
pub use zrml_swaps_runtime_api::SwapsApi as SwapsRuntimeApi;

#[rpc]
pub trait SwapsApi<BlockHash, PoolId, Hash, AccountId, Balance> {
    #[rpc(name = "swaps_poolSharesId")]
    fn pool_shares_id(
        &self,
        pool_id: PoolId,
        at: Option<BlockHash>,
    ) -> Result<Hash>;

    #[rpc(name = "swaps_poolAccountId")]
    fn pool_account_id(
        &self,
        pool_id: PoolId,
        at: Option<BlockHash>,
    ) -> Result<AccountId>;

    #[rpc(name = "swaps_getSpotPrice")]
    fn get_spot_price(
        &self,
        pool_id: PoolId,
        asset_in: Hash,
        asset_out: Hash,
        at: Option<BlockHash>,
    ) -> Result<Balance>;
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

impl<C, Block, PoolId, Hash, AccountId, Balance> SwapsApi<<Block as BlockT>::Hash, PoolId, Hash, AccountId, Balance>
    for Swaps<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: SwapsRuntimeApi<Block, PoolId, Hash, AccountId, Balance>,
    PoolId: Codec,
    Hash: Codec,
    AccountId: Codec,
    Balance: Codec,
{
    fn pool_shares_id(
        &self,
        pool_id: PoolId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Hash> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            //if the block hash is not supplied assume the best block
            self.client.info().best_hash));

        api.pool_shares_id(&at, pool_id)
            .map_err(|e| RpcError {
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

        api.pool_account_id(&at, pool_id)
            .map_err(|e| RpcError {
                code: ErrorCode::ServerError(Error::RuntimeError.into()),
                message: "Unable to get pool account identifier.".into(),
                data: Some(format!("{:?}", e).into()),
            })
    }

    fn get_spot_price(
        &self,
        pool_id: PoolId,
        asset_in: Hash,
        asset_out: Hash,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Balance> {
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
