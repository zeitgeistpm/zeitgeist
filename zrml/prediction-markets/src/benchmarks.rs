#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Config;
#[cfg(test)]
use crate::Pallet as PredictionMarket;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};

benchmarks! {
    first_bm {

    }: {}
}

impl_benchmark_test_suite!(
    PredictionMarket,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
