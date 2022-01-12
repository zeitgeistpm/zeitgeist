#[cfg(feature = "parachain")]
mod service_parachain;
#[cfg(not(feature = "parachain"))]
mod service_standalone;

#[cfg(feature = "parachain")]
use {
  sp_runtime::traits::BlakeTwo256,
  zeitgeist_primitives::types::{AccountId, Balance, Index, MarketId, PoolId},
  zeitgeist_runtime::opaque::Block,
};

#[cfg(feature = "parachain")]
pub use service_parachain::{new_full, new_partial};
#[cfg(not(feature = "parachain"))]
pub use service_standalone::{new_full, new_light, new_partial};

pub struct ExecutorDispatch;

impl sc_executor::NativeExecutionDispatch for ExecutorDispatch {
  #[cfg(feature = "runtime-benchmarks")]
  type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
  #[cfg(not(features = "runtime-benchmarks"))]
  type ExtendHostFunctions = ();

  fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
    zeitgeist_runtime::api::dispatch(method, data)
  }

  fn native_version() -> sc_executor::NativeVersion {
    zeitgeist_runtime::native_version()
  }
}

/// A set of APIs that polkadot-like runtimes must implement.
///
/// This trait has no methods or associated type. It is a concise marker for all the trait bounds
/// that it contains.
pub trait RuntimeApiCollection:
	sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
	+ sp_api::ApiExt<Block>
	+ sp_block_builder::BlockBuilder<Block>
	+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>
	+ pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
	+ sp_api::Metadata<Block>
	+ sp_offchain::OffchainWorkerApi<Block>
	+ sp_session::SessionKeys<Block>
	+ nimbus_primitives::AuthorFilterAPI<Block, nimbus_primitives::NimbusId>
	+ cumulus_primitives_core::CollectCollationInfo<Block>
  + zrml_swaps_rpc::SwapsRuntimeApi<Block, PoolId, AccountId, Balance, MarketId>
where
	<Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

impl<Api> RuntimeApiCollection for Api
where
	Api: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ sp_api::ApiExt<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>
		+ pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
		+ sp_api::Metadata<Block>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_session::SessionKeys<Block>
		+ nimbus_primitives::AuthorFilterAPI<Block, nimbus_primitives::NimbusId>
		+ cumulus_primitives_core::CollectCollationInfo<Block>
    + zrml_swaps_rpc::SwapsRuntimeApi<Block, PoolId, AccountId, Balance, MarketId>,
	<Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}