#[cfg(feature = "parachain")]
mod service_parachain;
#[cfg(not(feature = "parachain"))]
mod service_standalone;

use sp_runtime::traits::BlakeTwo256;
use zeitgeist_primitives::types::{AccountId, Balance, Index, MarketId, PoolId};
use zeitgeist_runtime::opaque::Block;

#[cfg(feature = "parachain")]
pub use service_parachain::{new_full, new_partial};
#[cfg(not(feature = "parachain"))]
pub use service_standalone::{new_full, new_light, new_partial};

pub struct ExecutorDispatch;

impl sc_executor::NativeExecutionDispatch for ExecutorDispatch {
    #[cfg(feature = "runtime-benchmarks")]
    type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
    #[cfg(not(feature = "runtime-benchmarks"))]
    type ExtendHostFunctions = ();

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        zeitgeist_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        zeitgeist_runtime::native_version()
    }
}

/// A set of common runtime APIs between standalone an parachain runtimes.
pub trait CommonRuntimeApiCollection:
    sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
    + sp_api::ApiExt<Block>
    + sp_block_builder::BlockBuilder<Block>
    + substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>
    + pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
    + sp_api::Metadata<Block>
    + sp_offchain::OffchainWorkerApi<Block>
    + sp_session::SessionKeys<Block>
    + zrml_swaps_rpc::SwapsRuntimeApi<Block, PoolId, AccountId, Balance, MarketId>
where
    <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

impl<Api> CommonRuntimeApiCollection for Api
where
    Api: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
        + sp_api::ApiExt<Block>
        + sp_block_builder::BlockBuilder<Block>
        + substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>
        + pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
        + sp_api::Metadata<Block>
        + sp_offchain::OffchainWorkerApi<Block>
        + sp_session::SessionKeys<Block>
        + zrml_swaps_rpc::SwapsRuntimeApi<Block, PoolId, AccountId, Balance, MarketId>,
    <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

cfg_if::cfg_if! {
    if #[cfg(feature = "parachain")] {
        /// Additional APIs for parachain runtimes
        pub trait AdditionalRuntimeApiCollection:
            sp_api::ApiExt<Block>
            + nimbus_primitives::AuthorFilterAPI<Block, nimbus_primitives::NimbusId>
            + cumulus_primitives_core::CollectCollationInfo<Block>
        where
            <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
        {
        }

        impl<Api> AdditionalRuntimeApiCollection for Api
        where
            Api: sp_api::ApiExt<Block>
                + nimbus_primitives::AuthorFilterAPI<Block, nimbus_primitives::NimbusId>
                + cumulus_primitives_core::CollectCollationInfo<Block>,
            <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
        {
        }
    } else {
        /// Additional APIs for standalone runtimes
        pub trait AdditionalRuntimeApiCollection:
            sp_api::ApiExt<Block>
            + sp_finality_grandpa::GrandpaApi<Block>
            + sp_consensus_aura::AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>
        where
            <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
        {
        }

        impl<Api> AdditionalRuntimeApiCollection for Api
        where
            Api: sp_api::ApiExt<Block>
                + sp_finality_grandpa::GrandpaApi<Block>
                + sp_consensus_aura::AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>,
            <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
        {
        }
    }
}
