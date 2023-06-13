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

use super::service::FullClient;

use sc_cli::Result;
use sc_client_api::BlockBackend;
use sc_executor::NativeExecutionDispatch;
use sp_core::{Encode, Pair};
use sp_inherents::{InherentData, InherentDataProvider};
use sp_keyring::Sr25519Keyring;
use sp_runtime::{OpaqueExtrinsic, SaturatedConversion};
use std::{sync::Arc, time::Duration};
use zeitgeist_primitives::types::{AccountId, Balance, Signature};

/// Generates extrinsics for the `benchmark overhead` command.
///
/// Note: Should only be used for benchmarking.
pub struct RemarksExtrinsicBuilder<RuntimeApi, Executor: NativeExecutionDispatch + 'static> {
    client: Arc<FullClient<RuntimeApi, Executor>>,
    is_zeitgeist: bool,
}

impl<RuntimeApi, Executor: NativeExecutionDispatch + 'static>
    RemarksExtrinsicBuilder<RuntimeApi, Executor>
{
    /// Creates a new [`Self`] from the given client.
    pub fn new(client: Arc<FullClient<RuntimeApi, Executor>>, is_zeitgeist: bool) -> Self {
        Self { client, is_zeitgeist }
    }
}

impl<RuntimeApi, Executor: NativeExecutionDispatch + 'static>
    frame_benchmarking_cli::ExtrinsicBuilder for RemarksExtrinsicBuilder<RuntimeApi, Executor>
{
    fn pallet(&self) -> &str {
        "system"
    }

    fn extrinsic(&self) -> &str {
        "remark"
    }

    fn build(&self, nonce: u32) -> std::result::Result<OpaqueExtrinsic, &'static str> {
        let acc = Sr25519Keyring::Bob.pair();

        #[cfg(feature = "with-zeitgeist-runtime")]
        if self.is_zeitgeist {
            return Ok(create_benchmark_extrinsic_zeitgeist(
                self.client.as_ref(),
                acc,
                zeitgeist_runtime::SystemCall::remark { remark: vec![] }.into(),
                nonce,
            )
            .into());
        }
        #[cfg(feature = "with-battery-station-runtime")]
        if !self.is_zeitgeist {
            return Ok(create_benchmark_extrinsic_battery_station(
                self.client.as_ref(),
                acc,
                battery_station_runtime::SystemCall::remark { remark: vec![] }.into(),
                nonce,
            )
            .into());
        }

        Err(crate::BATTERY_STATION_RUNTIME_NOT_AVAILABLE)
    }
}

/// Generates `Balances::TransferKeepAlive` extrinsics for the benchmarks.
///
/// Note: Should only be used for benchmarking.
pub struct TransferKeepAliveBuilder<RuntimeApi, Executor: NativeExecutionDispatch + 'static> {
    client: Arc<FullClient<RuntimeApi, Executor>>,
    dest: AccountId,
    value: Balance,
    is_zeitgeist: bool,
}

impl<RuntimeApi, Executor: NativeExecutionDispatch + 'static>
    TransferKeepAliveBuilder<RuntimeApi, Executor>
{
    /// Creates a new [`Self`] from the given client.
    pub fn new(
        client: Arc<FullClient<RuntimeApi, Executor>>,
        dest: AccountId,
        value: Balance,
        is_zeitgeist: bool,
    ) -> Self {
        Self { client, dest, value, is_zeitgeist }
    }
}

impl<RuntimeApi, Executor: NativeExecutionDispatch + 'static>
    frame_benchmarking_cli::ExtrinsicBuilder for TransferKeepAliveBuilder<RuntimeApi, Executor>
{
    fn pallet(&self) -> &str {
        "balances"
    }

    fn extrinsic(&self) -> &str {
        "transfer_keep_alive"
    }

    fn build(&self, nonce: u32) -> std::result::Result<OpaqueExtrinsic, &'static str> {
        let acc = Sr25519Keyring::Bob.pair();
        #[cfg(feature = "with-zeitgeist-runtime")]
        if self.is_zeitgeist {
            return Ok(create_benchmark_extrinsic_zeitgeist(
                self.client.as_ref(),
                acc,
                zeitgeist_runtime::BalancesCall::transfer_keep_alive {
                    dest: self.dest.clone().into(),
                    value: self.value,
                }
                .into(),
                nonce,
            )
            .into());
        }
        #[cfg(feature = "with-battery-station-runtime")]
        if !self.is_zeitgeist {
            return Ok(create_benchmark_extrinsic_battery_station(
                self.client.as_ref(),
                acc,
                battery_station_runtime::BalancesCall::transfer_keep_alive {
                    dest: self.dest.clone().into(),
                    value: self.value,
                }
                .into(),
                nonce,
            )
            .into());
        }

        Err(crate::BATTERY_STATION_RUNTIME_NOT_AVAILABLE)
    }
}

/// Creates a transaction using the given `call`.
///
/// Note: Should only be used for benchmarking.
#[cfg(feature = "with-zeitgeist-runtime")]
pub fn create_benchmark_extrinsic_zeitgeist<
    RuntimeApi,
    Executor: NativeExecutionDispatch + 'static,
>(
    client: &FullClient<RuntimeApi, Executor>,
    sender: sp_core::sr25519::Pair,
    call: zeitgeist_runtime::RuntimeCall,
    nonce: u32,
) -> zeitgeist_runtime::UncheckedExtrinsic {
    let genesis_hash = client.block_hash(0).ok().flatten().expect("Genesis block exists; qed");
    let best_hash = client.chain_info().best_hash;
    let best_block = client.chain_info().best_number;

    let period = zeitgeist_runtime::BlockHashCount::get()
        .checked_next_power_of_two()
        .map(|c| c / 2)
        .unwrap_or(2);
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
            zeitgeist_runtime::VERSION.spec_version,
            zeitgeist_runtime::VERSION.transaction_version,
            genesis_hash,
            best_hash,
            (),
            (),
            (),
        ),
    );
    let signature = raw_payload.using_encoded(|e| sender.sign(e));

    zeitgeist_runtime::UncheckedExtrinsic::new_signed(
        call,
        sp_runtime::AccountId32::from(sender.public()).into(),
        Signature::Sr25519(signature),
        extra,
    )
}

/// Creates a transaction using the given `call`.
///
/// Note: Should only be used for benchmarking.
#[cfg(feature = "with-battery-station-runtime")]
pub fn create_benchmark_extrinsic_battery_station<
    RuntimeApi,
    Executor: NativeExecutionDispatch + 'static,
>(
    client: &FullClient<RuntimeApi, Executor>,
    sender: sp_core::sr25519::Pair,
    call: battery_station_runtime::RuntimeCall,
    nonce: u32,
) -> battery_station_runtime::UncheckedExtrinsic {
    let genesis_hash = client.block_hash(0).ok().flatten().expect("Genesis block exists; qed");
    let best_hash = client.chain_info().best_hash;
    let best_block = client.chain_info().best_number;

    let period = battery_station_runtime::BlockHashCount::get()
        .checked_next_power_of_two()
        .map(|c| c / 2)
        .unwrap_or(2);
    let extra: battery_station_runtime::SignedExtra = (
        battery_station_runtime::CheckNonZeroSender::<battery_station_runtime::Runtime>::new(),
        battery_station_runtime::CheckSpecVersion::<battery_station_runtime::Runtime>::new(),
        battery_station_runtime::CheckTxVersion::<battery_station_runtime::Runtime>::new(),
        battery_station_runtime::CheckGenesis::<battery_station_runtime::Runtime>::new(),
        battery_station_runtime::CheckEra::<battery_station_runtime::Runtime>::from(
            sp_runtime::generic::Era::mortal(period, best_block.saturated_into()),
        ),
        battery_station_runtime::CheckNonce::<battery_station_runtime::Runtime>::from(nonce.into()),
        battery_station_runtime::CheckWeight::<battery_station_runtime::Runtime>::new(),
        pallet_transaction_payment::ChargeTransactionPayment::<battery_station_runtime::Runtime>::from(0),
    );

    let raw_payload = battery_station_runtime::SignedPayload::from_raw(
        call.clone(),
        extra.clone(),
        (
            (),
            battery_station_runtime::VERSION.spec_version,
            battery_station_runtime::VERSION.transaction_version,
            genesis_hash,
            best_hash,
            (),
            (),
            (),
        ),
    );
    let signature = raw_payload.using_encoded(|e| sender.sign(e));

    battery_station_runtime::UncheckedExtrinsic::new_signed(
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
