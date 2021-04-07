//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

pub use sc_rpc_api::DenyUnsafe;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_transaction_pool::TransactionPool;
use std::sync::Arc;
use zeitgeist_primitives::{AccountId, Balance, Hash, Index, MarketId, PoolId};
use zeitgeist_runtime::opaque::Block;

/// Full client dependencies.
pub struct FullDeps<C, P> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,
    /// Transaction pool instance.
    pub pool: Arc<P>,
}

/// Instantiate all full RPC extensions.
pub fn create_full<C, P>(deps: FullDeps<C, P>) -> jsonrpc_core::IoHandler<sc_rpc::Metadata>
where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: zrml_prediction_markets_rpc::PredictionMarketsRuntimeApi<Block, MarketId, Hash>,
    C::Api: zrml_swaps_rpc::SwapsRuntimeApi<Block, PoolId, Hash, AccountId, Balance, MarketId>,
    C::Api: BlockBuilder<Block>,
    P: TransactionPool + 'static,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
    use substrate_frame_rpc_system::{FullSystem, SystemApi};

    let mut io = jsonrpc_core::IoHandler::default();
    let FullDeps {
        client,
        pool,
        deny_unsafe,
    } = deps;

    io.extend_with(SystemApi::to_delegate(FullSystem::new(
        client.clone(),
        pool,
        deny_unsafe,
    )));

    io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
        client.clone(),
    )));

    io.extend_with(
        zrml_prediction_markets_rpc::PredictionMarketsApi::to_delegate(
            zrml_prediction_markets_rpc::PredictionMarkets::new(client.clone()),
        ),
    );

    io.extend_with(zrml_swaps_rpc::SwapsApi::to_delegate(
        zrml_swaps_rpc::Swaps::new(client.clone()),
    ));

    io
}
