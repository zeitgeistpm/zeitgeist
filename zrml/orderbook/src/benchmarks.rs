// Copyright 2023-2024 Forecasting Technologies LTD.
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
use frame_support::traits::UnfilteredDispatchable;
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::SaturatedConversion;
use zeitgeist_primitives::{constants::BASE, types::Asset};

fn generate_funded_account<T: Config>(
    seed: Option<u32>,
    asset: AssetOf<T>,
) -> Result<T::AccountId, &'static str> {
    let acc = if let Some(s) = seed { account("AssetHolder", 0, s) } else { whitelisted_caller() };
    T::AssetManager::deposit(asset, &acc, BASE.saturating_mul(1_000).saturated_into())?;
    Ok(acc)
}

fn order_common_parameters<T: Config>(
    seed: Option<u32>,
) -> Result<
    (MarketIdOf<T>, T::AccountId, Asset<MarketIdOf<T>>, BalanceOf<T>, BalanceOf<T>),
    &'static str,
> {
    let market = market_mock::<T>();
    let maker_asset = market.base_asset.into();
    let acc = generate_funded_account::<T>(seed, maker_asset)?;
    let maker_amount: BalanceOf<T> = BASE.saturating_mul(1_000).saturated_into();
    let taker_amount: BalanceOf<T> = BASE.saturating_mul(1_000).saturated_into();
    T::MarketCommons::push_market(market.clone()).unwrap();
    let market_id: MarketIdOf<T> = 0u32.into();
    Ok((market_id, acc, maker_asset, maker_amount, taker_amount))
}

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
        let market_id = 0u32.into();
        let taker_asset = Asset::CategoricalOutcome::<MarketIdOf<T>>(market_id, 0);
        let (caller, _, order_id) = place_default_order::<T>(None, taker_asset)?;
    }: remove_order(RawOrigin::Signed(caller), order_id)

    fill_order {
        let market_id = 0u32.into();
        let taker_asset = Asset::CategoricalOutcome::<MarketIdOf<T>>(market_id, 0);
        let (_, _, order_id) = place_default_order::<T>(Some(0), taker_asset)?;
        let caller = generate_funded_account::<T>(None, taker_asset)?;
        let maker_asset = T::MarketCommons::market(&market_id).unwrap().base_asset.into();
        let caller = generate_funded_account::<T>(None, maker_asset)?;
    }: fill_order(RawOrigin::Signed(caller), order_id, None)

    place_order {
        let (market_id, caller, maker_asset, maker_amount, taker_amount) =
            order_common_parameters::<T>(None)?;
        let taker_asset = Asset::CategoricalOutcome::<MarketIdOf<T>>(market_id, 0);
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
