#![cfg(feature = "runtime-benchmarks")]

use super::*;
#[cfg(test)]
use crate::Pallet as OrderBook;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{
    dispatch::UnfilteredDispatchable,
    traits::Currency,
};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::SaturatedConversion;
use zeitgeist_primitives::{Asset, BASE};

// Creates an account and gives it asset and currency.
// Returns `account`, `asset` and `amount/price` (for bid and asks)
fn order_common_parameters<T: Config>() -> 
    Result<(T::AccountId, Asset<T::MarketId>, BalanceOf::<T>), &'static str> {
    let caller = whitelisted_caller();
    let asset = Asset::CategoricalOutcome::<T::MarketId>(u128::MAX.saturated_into(), 0);
    let _ = T::Shares::deposit(asset, &caller, (u128::MAX).saturated_into())?;
    let _ = T::Currency::deposit_creating(&caller, (u128::MAX).saturated_into());
    let amount_and_price: BalanceOf::<T> = BASE.saturated_into();
    Ok((caller, asset, amount_and_price))
}

// Creates an order of type `order_type`
// Returns `account`, `asset` and `order_hash`
fn create_order<T: Config>(order_type: OrderSide) ->
    Result<(T::AccountId, Asset<T::MarketId>, T::Hash), &'static str> {
    let (caller, asset, amt_nd_prc) = order_common_parameters::<T>()?;
    let _ = Call::<T>::make_order(asset, order_type.clone(), amt_nd_prc, amt_nd_prc)
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;

    if order_type == OrderSide::Bid {
        let hash = Pallet::<T>::bids(asset).last().copied().ok_or("No bids found")?;
        return Ok((caller, asset, hash));
    } else {
        let hash = Pallet::<T>::asks(asset).last().copied().ok_or("No asks found")?;
        return Ok((caller, asset, hash));
    }
}

benchmarks! {
    make_order_bid {
        let (caller, asset, amt_nd_prc) = order_common_parameters::<T>()?;
    }: make_order(RawOrigin::Signed(caller), asset, OrderSide::Bid, amt_nd_prc, amt_nd_prc)

    make_order_ask {
        let (caller, asset, amt_nd_prc) = order_common_parameters::<T>()?;
    }: make_order(RawOrigin::Signed(caller), asset, OrderSide::Ask, amt_nd_prc, amt_nd_prc)

    cancel_order_bid {
        let (caller, asset, order_hash) = create_order::<T>(OrderSide::Bid)?;
    }: cancel_order(RawOrigin::Signed(caller), asset, order_hash)

    cancel_order_ask {
        let (caller, asset, order_hash) = create_order::<T>(OrderSide::Ask)?;
    }: cancel_order(RawOrigin::Signed(caller), asset, order_hash)
}

impl_benchmark_test_suite!(
    OrderBook,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
