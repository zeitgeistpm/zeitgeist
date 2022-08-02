use super::service::FullClient;

use sc_cli::Result;
use sc_client_api::BlockBackend;
use sc_executor::NativeExecutionDispatch;
use sp_core::{Encode, Pair};
use sp_inherents::{InherentData, InherentDataProvider};
use sp_keyring::Sr25519Keyring;
use sp_runtime::{OpaqueExtrinsic, SaturatedConversion};
use std::{sync::Arc, time::Duration};
use zeitgeist_primitives::{constants::BlockHashCount, types::Signature};
use zeitgeist_runtime::{SystemCall, UncheckedExtrinsic, VERSION};

/// Generates extrinsics for the `benchmark overhead` command.
///
/// Note: Should only be used for benchmarking.
pub struct BenchmarkExtrinsicBuilder<RuntimeApi, Executor: NativeExecutionDispatch + 'static> {
    client: Arc<FullClient<RuntimeApi, Executor>>,
}

impl<RuntimeApi, Executor: NativeExecutionDispatch + 'static>
    BenchmarkExtrinsicBuilder<RuntimeApi, Executor>
{
    /// Creates a new [`Self`] from the given client.
    pub fn new(client: Arc<FullClient<RuntimeApi, Executor>>) -> Self {
        Self { client }
    }
}

impl<RuntimeApi, Executor: NativeExecutionDispatch + 'static>
    frame_benchmarking_cli::ExtrinsicBuilder for BenchmarkExtrinsicBuilder<RuntimeApi, Executor>
{
    fn remark(&self, nonce: u32) -> std::result::Result<OpaqueExtrinsic, &'static str> {
        let acc = Sr25519Keyring::Bob.pair();
        let extrinsic: OpaqueExtrinsic = create_benchmark_extrinsic(
            self.client.as_ref(),
            acc,
            SystemCall::remark { remark: vec![] }.into(),
            nonce,
        )
        .into();

        Ok(extrinsic)
    }
}

/// Creates a transaction using the given `call`.
///
/// Note: Should only be used for benchmarking.
pub fn create_benchmark_extrinsic<RuntimeApi, Executor: NativeExecutionDispatch + 'static>(
    client: &FullClient<RuntimeApi, Executor>,
    sender: sp_core::sr25519::Pair,
    call: zeitgeist_runtime::Call,
    nonce: u32,
) -> UncheckedExtrinsic {
    let genesis_hash = client.block_hash(0).ok().flatten().expect("Genesis block exists; qed");
    let best_hash = client.chain_info().best_hash;
    let best_block = client.chain_info().best_number;

    let period =
        BlockHashCount::get().checked_next_power_of_two().map(|c| c / 2).unwrap_or(2) as u64;
    let extra: zeitgeist_runtime::SignedExtra = (
        zeitgeist_runtime::CheckNonZeroSender::<zeitgeist_runtime::Runtime>::new(),
        zeitgeist_runtime::CheckSpecVersion::<zeitgeist_runtime::Runtime>::new(),
        zeitgeist_runtime::CheckTxVersion::<zeitgeist_runtime::Runtime>::new(),
        zeitgeist_runtime::CheckGenesis::<zeitgeist_runtime::Runtime>::new(),
        zeitgeist_runtime::CheckEra::<zeitgeist_runtime::Runtime>::from(
            sp_runtime::generic::Era::mortal(period, best_block.saturated_into()),
        ),
        zeitgeist_runtime::CheckNonce::<zeitgeist_runtime::Runtime>::from(nonce.into()),
        zeitgeist_runtime::CheckWeight::<zeitgeist_runtime::Runtime>::new(),
        pallet_transaction_payment::ChargeTransactionPayment::<zeitgeist_runtime::Runtime>::from(0),
    );

    let raw_payload = zeitgeist_runtime::SignedPayload::from_raw(
        call.clone(),
        extra.clone(),
        (
            (),
            VERSION.spec_version,
            VERSION.transaction_version,
            genesis_hash,
            best_hash,
            (),
            (),
            (),
        ),
    );
    let signature = raw_payload.using_encoded(|e| sender.sign(e));

    UncheckedExtrinsic::new_signed(
        call,
        sp_runtime::AccountId32::from(sender.public()).into(),
        Signature::Sr25519(signature),
        extra,
    )
}

/// Generates inherent data for the `benchmark overhead` command.
///
/// Note: Should only be used for benchmarking.
pub fn inherent_benchmark_data() -> Result<InherentData> {
    let mut inherent_data = InherentData::new();
    let d = Duration::from_millis(0);
    let timestamp = sp_timestamp::InherentDataProvider::new(d.into());

    timestamp
        .provide_inherent_data(&mut inherent_data)
        .map_err(|e| format!("creating inherent data: {:?}", e))?;
    Ok(inherent_data)
}
