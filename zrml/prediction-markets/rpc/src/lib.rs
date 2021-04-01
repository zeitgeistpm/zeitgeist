//! RPC interface for the Prediction Markets pallet.

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;
use zeitgeist_primitives::Asset;

pub use zrml_prediction_markets_runtime_api::PredictionMarketsApi as PredictionMarketsRuntimeApi;

#[rpc]
pub trait PredictionMarketsApi<BlockHash, MarketId, Hash> {
    #[rpc(name = "predictionMarkets_marketOutcomeShareId")]
    fn market_outcome_share_id(
        &self,
        market_id: MarketId,
        outcome: u16,
        at: Option<BlockHash>,
    ) -> Result<Asset<Hash, MarketId>>;
}

/// A struct that implements the [`PredictionMarketsApi`].
pub struct PredictionMarkets<C, B> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>,
}

impl<C, B> PredictionMarkets<C, B> {
    /// Create a new `PredictionMarkets` with the given reference to
    /// the client.
    pub fn new(client: Arc<C>) -> Self {
        PredictionMarkets {
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

impl<C, Block, MarketId, Hash> PredictionMarketsApi<<Block as BlockT>::Hash, MarketId, Hash>
    for PredictionMarkets<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: PredictionMarketsRuntimeApi<Block, MarketId, Hash>,
    MarketId: Codec,
    Hash: Codec,
{
    fn market_outcome_share_id(
        &self,
        market_id: MarketId,
        outcome: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Asset<Hash, MarketId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume
            // best block.
            self.client.info().best_hash));

        api.market_outcome_share_id(&at, market_id, outcome)
            .map_err(|e| RpcError {
                code: ErrorCode::ServerError(Error::RuntimeError.into()),
                message: "Unable to get the market outcome share identifier.".into(),
                data: Some(format!("{:?}", e).into()),
            })
    }
}
