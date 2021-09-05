#![allow(
    // Auto-generated code is a no man's land
    clippy::integer_arithmetic
)]
#![cfg(feature = "runtime-benchmarks")]

use crate::pallet::{BalanceOf, Call, Config, Pallet};
#[cfg(test)]
use crate::Pallet as LiquidityMining;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;
use sp_runtime::traits::Bounded;

benchmarks! {
    set_per_block_distribution {
        let balance = BalanceOf::<T>::max_value();
    }: set_per_block_distribution(RawOrigin::Root, balance)
}

impl_benchmark_test_suite!(
    LiquidityMining,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
