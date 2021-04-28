#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Config;
#[cfg(test)]
use crate::Pallet as OrderBook;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};


benchmarks! {
    dummy_bench {

    }: {}
}

impl_benchmark_test_suite!(
    OrderBook,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
