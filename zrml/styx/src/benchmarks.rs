#![allow(
  // Auto-generated code is a no man's land
  clippy::integer_arithmetic
)]
#![allow(clippy::type_complexity)]
#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Config;
#[cfg(test)]
use crate::Pallet as Styx;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{
    dispatch::UnfilteredDispatchable,
    traits::{Currency, EnsureOrigin},
};
use frame_system::RawOrigin;
use sp_runtime::SaturatedConversion;
use zeitgeist_primitives::constants::BASE;

benchmarks! {
  cross {
      let caller: T::AccountId = whitelisted_caller();
      let balance = (90_000_000 * BASE).saturated_into();
      T::Currency::deposit_creating(&caller, balance);
  }: _(RawOrigin::Signed(caller))

  set_burn_amount {
      let origin = T::SetBurnAmountOrigin::successful_origin();
      let caller: T::AccountId = whitelisted_caller();
      let balance = 100_000_000_000_000u128.saturated_into();
      let amount = 200_000_000_000u128.saturated_into();
      T::Currency::deposit_creating(&caller, balance);
      let call = Call::<T>::set_burn_amount { amount };
  }: { call.dispatch_bypass_filter(origin)? }
}

impl_benchmark_test_suite!(Styx, crate::mock::ExtBuilder::default().build(), crate::mock::Runtime);
