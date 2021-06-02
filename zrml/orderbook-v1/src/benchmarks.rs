#![cfg(feature = "runtime-benchmarks")]

use super::*;
#[cfg(test)]
use crate::Pallet as OrderBook;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{dispatch::UnfilteredDispatchable, traits::Currency};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::SaturatedConversion;
use zeitgeist_primitives::{constants::BASE, types::Asset};

// Takes a `seed` and returns an account. Use None to generate a whitelisted caller
fn generate_funded_account<T: Config>(seed: Option<u32>) -> Result<T::AccountId, &'static str> {
    let acc: T::AccountId;

    if let Some(s) = seed {
        acc = account("AssetHolder", 0, s);
    } else {
        acc = whitelisted_caller();
    }

    let asset = Asset::CategoricalOutcome::<T::MarketId>(0u32.into(), 0);
    let _ = T::Shares::deposit(asset, &acc, BASE.saturating_mul(1_000).saturated_into())?;
    let _ = T::Currency::deposit_creating(&acc, BASE.saturating_mul(1_000).saturated_into());
    Ok(acc)
}

// Creates an account and gives it asset and currency. `seed` specifies the account seed,
// None will return a whitelisted account
// Returns `account`, `asset`, `amount` and `price`
fn order_common_parameters<T: Config>(
    seed: Option<u32>,
) -> Result<(T::AccountId, Asset<T::MarketId>, BalanceOf<T>, BalanceOf<T>), &'static str> {
    let acc = generate_funded_account::<T>(seed)?;
    let asset = Asset::CategoricalOutcome::<T::MarketId>(0u32.into(), 0);
    let amt: BalanceOf<T> = BASE.saturated_into();
    let prc: BalanceOf<T> = 1u32.into();
    Ok((acc, asset, amt, prc))
}

// Creates an order of type `order_type`. `seed` specifies the account seed,
// None will return a whitelisted account
// Returns `account`, `asset` and `order_hash`
fn create_order<T: Config>(
    order_type: OrderSide,
    seed: Option<u32>,
) -> Result<(T::AccountId, Asset<T::MarketId>, T::Hash), &'static str> {
    let (acc, asset, amt, prc) = order_common_parameters::<T>(seed)?;
    let _ = Call::<T>::make_order(asset, order_type.clone(), amt, prc)
        .dispatch_bypass_filter(RawOrigin::Signed(acc.clone()).into())?;

    if order_type == OrderSide::Bid {
        let hash = Pallet::<T>::bids(asset)
            .last()
            .copied()
            .ok_or("No bids found")?;
        Ok((acc, asset, hash))
    } else {
        let hash = Pallet::<T>::asks(asset)
            .last()
            .copied()
            .ok_or("No asks found")?;
        Ok((acc, asset, hash))
    }
}

benchmarks! {
    cancel_order_ask {
        let (caller, asset, order_hash) = create_order::<T>(OrderSide::Ask, None)?;
    }: cancel_order(RawOrigin::Signed(caller), asset, order_hash)

    cancel_order_bid {
        let (caller, asset, order_hash) = create_order::<T>(OrderSide::Bid, None)?;
    }: cancel_order(RawOrigin::Signed(caller), asset, order_hash)

    fill_order_ask {
        let caller = generate_funded_account::<T>(None)?;
        let (_, _, order_hash) = create_order::<T>(OrderSide::Ask, Some(0))?;
    }: fill_order(RawOrigin::Signed(caller), order_hash)

    fill_order_bid {
        let caller = generate_funded_account::<T>(None)?;
        let (_, _, order_hash) = create_order::<T>(OrderSide::Bid, Some(0))?;
    }: fill_order(RawOrigin::Signed(caller), order_hash)

    make_order_ask {
        let (caller, asset, amt, prc) = order_common_parameters::<T>(None)?;
    }: make_order(RawOrigin::Signed(caller), asset, OrderSide::Ask, amt, prc)

    make_order_bid {
        let (caller, asset, amt, prc) = order_common_parameters::<T>(None)?;
    }: make_order(RawOrigin::Signed(caller), asset, OrderSide::Bid, amt, prc)
}

impl_benchmark_test_suite!(
    OrderBook,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
