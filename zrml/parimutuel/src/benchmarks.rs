// Copyright 2022-2023 Forecasting Technologies LTD.
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

use crate::{Pallet as Parimutuel, *};
use core::ops::RangeInclusive;
use frame_benchmarking::v2::*;
use frame_support::traits::Get;
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use zeitgeist_primitives::types::{
    Asset, MarketStatus, MarketType, Outcome, OutcomeReport, ScalarPosition,
};
use zrml_market_commons::MarketCommonsPalletApi;

fn setup_market<T: Config>(market_type: MarketType) -> MarketIdOf<T> {
    let market_id = 0u32.into();
    let mut market = market_mock::<T>();
    market.market_type = market_type;
    market.status = MarketStatus::Active;
    T::MarketCommons::push_market(market.clone()).unwrap();
    market_id
}

fn buy_asset<T: Config>(
    market_id: MarketIdOf<T>,
    asset: AssetOf<T>,
    buyer: &T::AccountId,
    amount: BalanceOf<T>,
) {
    let market = T::MarketCommons::market(&market_id).unwrap();
    T::AssetManager::deposit(market.base_asset, buyer, amount).unwrap();
    Parimutuel::<T>::buy(RawOrigin::Signed(buyer.clone()).into(), asset, amount).unwrap();
}

#[benchmarks]
mod benchmarks_parimutuel {
    use super::*;

    #[benchmark]
    fn buy() {
        let buyer = whitelisted_caller();

        let market_id = setup_market::<T>(MarketType::Categorical(T::MaxCategories::get()));

        let amount = T::MinBetSize::get();
        let asset = Asset::ParimutuelShare(Outcome::CategoricalOutcome(market_id, 0u16));

        let market = T::MarketCommons::market(&market_id).unwrap();
        T::AssetManager::deposit(market.base_asset, &buyer, amount).unwrap();

        #[extrinsic_call]
        buy(RawOrigin::Signed(buyer), asset, amount);
    }

    #[benchmark]
    fn claim_rewards_categorical() {
        // max category index is worst case
        let market_id = setup_market::<T>(MarketType::Categorical(T::MaxCategories::get()));

        let winner = whitelisted_caller();
        let winner_asset = Asset::ParimutuelShare(Outcome::CategoricalOutcome(market_id, 0u16));
        let winner_amount = T::MinBetSize::get() + T::MinBetSize::get();
        buy_asset::<T>(market_id, winner_asset, &winner, winner_amount);

        let loser = whitelisted_caller();
        let loser_asset = Asset::ParimutuelShare(Outcome::CategoricalOutcome(market_id, 1u16));
        let loser_amount = T::MinBetSize::get();
        buy_asset::<T>(market_id, loser_asset, &loser, loser_amount);

        T::MarketCommons::mutate_market(&market_id, |market| {
            market.status = MarketStatus::Resolved;
            market.resolved_outcome = Some(OutcomeReport::Categorical(0u16));
            Ok(())
        })?;

        #[extrinsic_call]
        claim_rewards(RawOrigin::Signed(winner), market_id);
    }

    #[benchmark]
    fn claim_rewards_scalar() {
        let range: RangeInclusive<u128> = 0..=100;
        let market_id = setup_market::<T>(MarketType::Scalar(range));

        let winner = whitelisted_caller();
        let winner_asset =
            Asset::ParimutuelShare(Outcome::ScalarOutcome(market_id, ScalarPosition::Long));
        let winner_amount = T::MinBetSize::get() + T::MinBetSize::get();
        buy_asset::<T>(market_id, winner_asset, &winner, winner_amount);
        // buy both Scalar positions to get the worst case
        let winner_asset =
            Asset::ParimutuelShare(Outcome::ScalarOutcome(market_id, ScalarPosition::Short));
        let winner_amount = T::MinBetSize::get();
        buy_asset::<T>(market_id, winner_asset, &winner, winner_amount);

        let loser = whitelisted_caller();
        let loser_asset =
            Asset::ParimutuelShare(Outcome::ScalarOutcome(market_id, ScalarPosition::Short));
        let loser_amount = T::MinBetSize::get();
        buy_asset::<T>(market_id, loser_asset, &loser, loser_amount);

        T::MarketCommons::mutate_market(&market_id, |market| {
            market.status = MarketStatus::Resolved;
            // pot distributes to 75% LONG and 25% SHORT
            market.resolved_outcome = Some(OutcomeReport::Scalar(75u128));
            Ok(())
        })?;

        #[extrinsic_call]
        claim_rewards(RawOrigin::Signed(winner), market_id);
    }

    #[benchmark]
    fn refund_pot() {
        // max category index is worst case
        let market_id = setup_market::<T>(MarketType::Categorical(T::MaxCategories::get()));

        let loser_0 = whitelisted_caller();
        let loser_0_index = 0u16;
        let loser_0_asset =
            Asset::ParimutuelShare(Outcome::CategoricalOutcome(market_id, loser_0_index));
        let loser_0_amount = T::MinBetSize::get() + T::MinBetSize::get();
        buy_asset::<T>(market_id, loser_0_asset, &loser_0, loser_0_amount);

        let loser_1 = whitelisted_caller();
        let loser_1_index = 1u16;
        let loser_1_asset =
            Asset::ParimutuelShare(Outcome::CategoricalOutcome(market_id, loser_1_index));
        let loser_1_amount = T::MinBetSize::get();
        buy_asset::<T>(market_id, loser_1_asset, &loser_1, loser_1_amount);

        T::MarketCommons::mutate_market(&market_id, |market| {
            market.status = MarketStatus::Resolved;
            let resolved_index = 9u16;
            let resolved_outcome = OutcomeReport::Categorical(resolved_index);
            assert_ne!(resolved_index, loser_0_index);
            assert_ne!(resolved_index, loser_1_index);
            let resolved_asset =
                Asset::ParimutuelShare(Outcome::CategoricalOutcome(market_id, resolved_index));
            let resolved_issuance_asset = T::AssetManager::total_issuance(resolved_asset);
            assert!(resolved_issuance_asset.is_zero());
            market.resolved_outcome = Some(resolved_outcome);
            Ok(())
        })?;

        #[extrinsic_call]
        refund_pot(RawOrigin::Signed(loser_0), loser_0_asset);
    }

    impl_benchmark_test_suite!(
        Parimutuel,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime
    );
}
