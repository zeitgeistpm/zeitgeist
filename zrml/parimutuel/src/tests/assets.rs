// Copyright 2024 Forecasting Technologies LTD.
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

use crate::{mock::*, utils::*, *};
use core::ops::RangeInclusive;
use frame_support::{
    assert_noop, assert_ok,
    pallet_prelude::Weight,
    traits::{
        fungibles::{Create, Inspect},
        OnIdle,
    },
};
use orml_traits::MultiCurrency;
use sp_runtime::Percent;
use test_case::test_case;
use zeitgeist_primitives::{
    traits::MarketTransitionApi,
    types::{Asset, MarketStatus, MarketType, OutcomeReport, ParimutuelAsset, ScoringRule},
};
use zrml_market_commons::{Error as MError, Markets};

use frame_support::{
    pallet_prelude::DispatchError,
    storage::{with_transaction, TransactionOutcome},
};

#[test]
fn created_after_market_activation() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market.clone());

        let _ = with_transaction(|| {
            assert_ok!(Parimutuel::on_activation(&market_id));
            TransactionOutcome::Commit(Ok::<(), DispatchError>(()))
        });

        for asset in market.outcome_assets(market_id) {
            assert!(<Runtime as Config>::AssetCreator::asset_exists(asset.into()));
        }
    });
}

#[test]
fn destroyed_after_claim() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let winner_asset = ParimutuelAsset::Share(market_id, 0u16);
        AssetRouter::create(winner_asset.into(), Default::default(), true, 1).unwrap();
        let winner_amount = 20 * <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(ALICE), winner_asset, winner_amount));

        let mut market = Markets::<Runtime>::get(market_id).unwrap();
        market.status = MarketStatus::Resolved;
        market.resolved_outcome = Some(OutcomeReport::Categorical(0u16));
        Markets::<Runtime>::insert(market_id, market.clone());

        assert_ok!(Parimutuel::claim_rewards(RuntimeOrigin::signed(ALICE), market_id));
        <Runtime as Config>::AssetDestroyer::on_idle(System::block_number(), Weight::MAX);
        assert!(!AssetRouter::asset_exists(winner_asset.into()));
    });
}

#[test]
fn destroyed_losing_after_resolution_with_winner() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);
        let _ = with_transaction(|| {
            assert_ok!(Parimutuel::on_activation(&market_id));
            TransactionOutcome::Commit(Ok::<(), DispatchError>(()))
        });

        let winner_asset = ParimutuelAsset::Share(market_id, 0u16);
        let winner_amount = 20 * <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(ALICE), winner_asset, winner_amount));

        let mut market = Markets::<Runtime>::get(market_id).unwrap();
        market.status = MarketStatus::Resolved;
        market.resolved_outcome = Some(OutcomeReport::Categorical(0u16));
        Markets::<Runtime>::insert(market_id, market.clone());

        let _ = with_transaction(|| {
            assert_ok!(Parimutuel::on_resolution(&market_id));
            TransactionOutcome::Commit(Ok::<(), DispatchError>(()))
        });
        <Runtime as Config>::AssetDestroyer::on_idle(System::block_number(), Weight::MAX);
        assert!(<Runtime as Config>::AssetCreator::asset_exists(winner_asset.into()));

        for asset in market
            .outcome_assets(market_id)
            .iter()
            .filter(|a| Asset::from(**a) != Asset::from(winner_asset))
        {
            assert!(
                !<Runtime as Config>::AssetCreator::asset_exists((*asset).into()),
                "Asset {:?} seems to exist?",
                asset
            );
        }
    });
}

#[test]
#[should_panic(expected = "Resolved market with id 0 does not have a resolved outcome")]
fn no_resolved_outcome_is_catched() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let winner_asset = ParimutuelAsset::Share(market_id, 0u16);
        AssetRouter::create(winner_asset.into(), Default::default(), true, 1).unwrap();
        let winner_amount = 20 * <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(ALICE), winner_asset, winner_amount));

        let mut market = Markets::<Runtime>::get(market_id).unwrap();
        market.status = MarketStatus::Resolved;
        market.resolved_outcome = None;
        Markets::<Runtime>::insert(market_id, market.clone());

        let _ = with_transaction(|| {
            let _ = Parimutuel::on_resolution(&market_id);
            TransactionOutcome::Commit(Ok::<(), DispatchError>(()))
        });
    });
}

/*
#[test]
fn destroyed_after_resolution_without_winner() {
    ExtBuilder::default().build().execute_with(|| {
        // TODO
    });
}

#[test]
fn destroyed_after_refund() {
    ExtBuilder::default().build().execute_with(|| {
        // TODO
    });
}

#[test]
#[should_panic(expected = "Can't destroy losing outcome asset: TODO")]
fn deprecated_market_assets_are_not_destroyed() {
    ExtBuilder::default().build().execute_with(|| {
        // TODO
    });
}
*/