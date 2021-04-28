#![cfg(feature = "runtime-benchmarks")]

use super::*;
#[cfg(test)]
use crate::Pallet as OrderBook;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::SaturatedConversion;
use zeitgeist_primitives::{Asset, BASE};

/*
fn order_common_parameters<T: Config> -> (T::AccountId, BalanceOf::<T>, BalanceOf::<T>, BalanceOf::<T>)

}*/

benchmarks! {
    make_order_bid {
        let caller = whitelisted_caller();
        let asset = Asset::CategoricalOutcome::<T::MarketId>(u128::MAX.saturated_into(), 0);
        let _ = T::Shares::deposit(asset, &caller, (u128::MAX).saturated_into())?;
        let _ = T::Currency::deposit_creating(&caller, (u128::MAX).saturated_into());
        let amount_and_price: BalanceOf::<T> = BASE.saturated_into();
    }: make_order(RawOrigin::Signed(caller), asset, OrderSide::Bid, amount_and_price, amount_and_price)

    make_order_ask {
        let caller = whitelisted_caller();
        let asset = Asset::CategoricalOutcome::<T::MarketId>(u128::MAX.saturated_into(), 0);
        let _ = T::Shares::deposit(asset, &caller, (u128::MAX).saturated_into())?;
        let _ = T::Currency::deposit_creating(&caller, (u128::MAX).saturated_into());
        let amount_and_price: BalanceOf::<T> = BASE.saturated_into();
    }: make_order(RawOrigin::Signed(caller), asset, OrderSide::Ask, amount_and_price, amount_and_price)

    /*
    cancel_order_bid {
        let caller = whitelisted_caller();
        let asset = Asset::CategoricalOutcome::<T::MarketId>(u128::MAX.saturated_into(), 0);
        let _ = T::Shares::deposit(asset, &caller, (u128::MAX).saturated_into())?;
        let _ = T::Currency::deposit_creating(&caller, (u128::MAX).saturated_into());
        let amount_and_price: BalanceOf::<T> = BASE.saturated_into();
    }: make_order(RawOrigin::Signed(caller), asset, OrderSide::Bid, amount_and_price, amount_and_price)

    cancel_order_ask {
        let caller = whitelisted_caller();
        let asset = Asset::CategoricalOutcome::<T::MarketId>(u128::MAX.saturated_into(), 0);
        let _ = T::Shares::deposit(asset, &caller, (u128::MAX).saturated_into())?;
        let _ = T::Currency::deposit_creating(&caller, (u128::MAX).saturated_into());
        let amount_and_price: BalanceOf::<T> = BASE.saturated_into();
    }: make_order(RawOrigin::Signed(caller), asset, OrderSide::Ask, amount_and_price, amount_and_price)
    */
}

impl_benchmark_test_suite!(
    OrderBook,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
