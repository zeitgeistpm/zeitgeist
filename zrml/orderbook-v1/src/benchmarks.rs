// Copyright 2023 Forecasting Technologies LTD.
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

#![allow(
    // Auto-generated code is a no man's land
    clippy::arithmetic_side_effects
)]
#![cfg(feature = "runtime-benchmarks")]
#![allow(clippy::type_complexity)]

use super::*;
use crate::utils::market_mock;
#[cfg(test)]
use crate::Pallet as OrderBook;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::dispatch::UnfilteredDispatchable;
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::SaturatedConversion;
use zeitgeist_primitives::{constants::BASE, types::Asset};

// Takes a `seed` and returns an account. Use None to generate a whitelisted caller
fn generate_funded_account<T: Config>(seed: Option<u32>) -> Result<T::AccountId, &'static str> {
    let acc = if let Some(s) = seed { account("AssetHolder", 0, s) } else { whitelisted_caller() };

    let outcome_asset = Asset::CategoricalOutcome::<MarketIdOf<T>>(0u32.into(), 0);
    T::AssetManager::deposit(outcome_asset, &acc, BASE.saturating_mul(1_000).saturated_into())?;
    let _ = T::AssetManager::deposit(Asset::Ztg, &acc, BASE.saturating_mul(1_000).saturated_into());
    Ok(acc)
}

// Creates an account and gives it asset and currency. `seed` specifies the account seed,
// None will return a whitelisted account
// Returns `account`, `asset`, `outcome_asset_amount` and `base_asset_amount`
fn order_common_parameters<T: Config>(
    seed: Option<u32>,
) -> Result<
    (T::AccountId, Asset<MarketIdOf<T>>, BalanceOf<T>, BalanceOf<T>, MarketIdOf<T>),
    &'static str,
> {
    let acc = generate_funded_account::<T>(seed)?;
    let outcome_asset = Asset::CategoricalOutcome::<MarketIdOf<T>>(0u32.into(), 0);
    let outcome_asset_amount: BalanceOf<T> = BASE.saturated_into();
    let base_asset_amount: BalanceOf<T> = 100u32.into();
    let market_id: MarketIdOf<T> = 0u32.into();
    let market = market_mock::<T>();
    T::MarketCommons::push_market(market.clone()).unwrap();
    Ok((acc, outcome_asset, outcome_asset_amount, base_asset_amount, market_id))
}

// Creates an order of type `order_type`. `seed` specifies the account seed,
// None will return a whitelisted account
// Returns `account`, `asset`, `order_id`
fn place_order<T: Config>(
    order_type: OrderSide,
    seed: Option<u32>,
) -> Result<(T::AccountId, MarketIdOf<T>, OrderId), &'static str> {
    let (acc, outcome_asset, outcome_asset_amount, base_asset_amount, market_id) =
        order_common_parameters::<T>(seed)?;

    let order_id = <NextOrderId<T>>::get();
    let _ = Call::<T>::place_order {
        market_id,
        outcome_asset,
        side: order_type.clone(),
        outcome_asset_amount,
        base_asset_amount,
    }
    .dispatch_bypass_filter(RawOrigin::Signed(acc.clone()).into())?;

    Ok((acc, market_id, order_id))
}

benchmarks! {
    remove_order_ask {
        let (caller, _, order_id) = place_order::<T>(OrderSide::Ask, None)?;
    }: remove_order(RawOrigin::Signed(caller), order_id)

    remove_order_bid {
        let (caller, _, order_id) = place_order::<T>(OrderSide::Bid, None)?;
    }: remove_order(RawOrigin::Signed(caller), order_id)

    fill_order_ask {
        let caller = generate_funded_account::<T>(None)?;
        let (_, _, order_id) = place_order::<T>(OrderSide::Ask, Some(0))?;
    }: fill_order(RawOrigin::Signed(caller), order_id, None)

    fill_order_bid {
        let caller = generate_funded_account::<T>(None)?;
        let (_, _, order_id) = place_order::<T>(OrderSide::Bid, Some(0))?;
    }: fill_order(RawOrigin::Signed(caller), order_id, None)

    place_order_ask {
        let (caller, outcome_asset, outcome_asset_amount, base_asset_amount, market_id) = order_common_parameters::<T>(None)?;
    }: place_order(RawOrigin::Signed(caller), market_id, outcome_asset, OrderSide::Ask, outcome_asset_amount, base_asset_amount)

    place_order_bid {
        let (caller, outcome_asset, outcome_asset_amount, base_asset_amount, market_id) = order_common_parameters::<T>(None)?;
    }: place_order(RawOrigin::Signed(caller), market_id, outcome_asset, OrderSide::Bid, outcome_asset_amount, base_asset_amount)

    impl_benchmark_test_suite!(
        OrderBook,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime,
    );
}
