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
use crate::{utils::market_mock, Pallet as Orderbook};
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::dispatch::UnfilteredDispatchable;
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::SaturatedConversion;
use zeitgeist_primitives::{constants::BASE, types::Asset};

// Takes a `seed` and returns an account. Use None to generate a whitelisted caller
fn generate_funded_account<T: Config>(
    seed: Option<u32>,
    asset: AssetOf<T>,
) -> Result<T::AccountId, &'static str> {
    let acc = if let Some(s) = seed { account("AssetHolder", 0, s) } else { whitelisted_caller() };
    T::AssetManager::deposit(asset, &acc, BASE.saturating_mul(1_000).saturated_into())?;
    Ok(acc)
}

// Creates an account and gives it asset and currency. `seed` specifies the account seed,
// None will return a whitelisted account
// Returns `account`, `asset`, `outcome_asset_amount` and `base_asset_amount`
fn order_common_parameters<T: Config>(
    seed: Option<u32>,
) -> Result<
    (MarketIdOf<T>, T::AccountId, Asset<MarketIdOf<T>>, BalanceOf<T>, BalanceOf<T>),
    &'static str,
> {
    let market = market_mock::<T>();
    let maker_asset = market.base_asset;
    let acc = generate_funded_account::<T>(seed, maker_asset)?;
    let maker_amount: BalanceOf<T> = 100u32.into();

    let taker_amount: BalanceOf<T> = BASE.saturated_into();
    T::MarketCommons::push_market(market.clone()).unwrap();
    let market_id: MarketIdOf<T> = 0u32.into();
    Ok((market_id, acc, maker_asset, maker_amount, taker_amount))
}

// Creates an order of type `order_type`. `seed` specifies the account seed,
// None will return a whitelisted account
// Returns `account`, `asset`, `order_id`
fn place_default_order<T: Config>(
    seed: Option<u32>,
    taker_asset: AssetOf<T>,
) -> Result<(T::AccountId, MarketIdOf<T>, OrderId), &'static str> {
    let (market_id, acc, maker_asset, maker_amount, taker_amount) =
        order_common_parameters::<T>(seed)?;

    let order_id = <NextOrderId<T>>::get();
    let _ =
        Call::<T>::place_order { market_id, maker_asset, maker_amount, taker_asset, taker_amount }
            .dispatch_bypass_filter(RawOrigin::Signed(acc.clone()).into())?;

    Ok((acc, market_id, order_id))
}

benchmarks! {
    remove_order {
        let taker_asset = Asset::CategoricalOutcome::<MarketIdOf<T>>(0u32.into(), 0);
        let (caller, _, order_id) = place_default_order::<T>(None, taker_asset)?;
    }: remove_order(RawOrigin::Signed(caller), order_id)

    fill_order {
        let taker_asset = Asset::CategoricalOutcome::<MarketIdOf<T>>(0u32.into(), 0);
        let caller = generate_funded_account::<T>(None, taker_asset)?;
        let (_, market_id, order_id) = place_default_order::<T>(Some(0), taker_asset)?;
        let maker_asset = T::MarketCommons::market(&market_id).unwrap().base_asset;
        let caller = generate_funded_account::<T>(None, maker_asset)?;
    }: fill_order(RawOrigin::Signed(caller), order_id, None)

    place_order {
        let taker_asset = Asset::CategoricalOutcome::<MarketIdOf<T>>(0u32.into(), 0);
        let (market_id, caller, maker_asset, maker_amount, taker_amount) =
            order_common_parameters::<T>(None)?;
    }: {
        Orderbook::<T>::place_order(
            RawOrigin::Signed(caller).into(),
            market_id,
            maker_asset,
            maker_amount,
            taker_asset,
            taker_amount,
        )?;
    }

    impl_benchmark_test_suite!(
        Orderbook,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime,
    );
}
