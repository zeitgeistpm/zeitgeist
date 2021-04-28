#![cfg(feature = "runtime-benchmarks")]

use super::{
    pallet::*,
    OrderSide,
};
#[cfg(test)]
use crate::Pallet as OrderBook;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::SaturatedConversion;
use zeitgeist_primitives::{Asset, BASE};

benchmarks! {
    make_order {
        let caller = whitelisted_caller();
        let asset = Asset::CategoricalOutcome::<T::MarketId>(u128::MAX.saturated_into(), 0);
        let _ = T::Shares::deposit(asset, &caller, (u128::MAX).saturated_into())?;
        let _ = T::Currency::deposit_creating(&caller, (u128::MAX).saturated_into());
        let amount_and_price: BalanceOf::<T> = BASE.saturated_into();
    }: _(RawOrigin::Signed(caller), asset, OrderSide::Bid, amount_and_price, amount_and_price)
}

impl_benchmark_test_suite!(
    OrderBook,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
