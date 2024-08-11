// Copyright 2022-2024 Forecasting Technologies LTD.
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

use super::*;

use crate::MarketIdsForEdit;

// TODO(#1239) MarketEditNotRequested
// TODO(#1239) MarketDoesNotExist
// TODO(#1239) InvalidMarketStatus
// TODO(#1239) All failures that need to be ensured for `create_market`

#[test]
fn only_creator_can_edit_market() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            0..2,
            ScoringRule::AmmCdaHybrid,
        );

        // make sure it's in status proposed
        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        let edit_reason = vec![0_u8; <Runtime as Config>::MaxEditReasonLen::get() as usize];

        // Now it should work for the designated origin.
        assert_ok!(PredictionMarkets::request_edit(
            RuntimeOrigin::signed(RequestEditOrigin::get()),
            0,
            edit_reason
        ));

        assert!(MarketIdsForEdit::<Runtime>::contains_key(0));

        // ALICE is market creator through simple_create_categorical_market
        assert_noop!(
            PredictionMarkets::edit_market(
                RuntimeOrigin::signed(BOB),
                Asset::Ztg,
                0,
                CHARLIE,
                MarketPeriod::Block(0..2),
                get_deadlines(),
                gen_metadata(2),
                MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
                Some(MarketDisputeMechanism::SimpleDisputes),
                ScoringRule::AmmCdaHybrid
            ),
            Error::<Runtime>::EditorNotCreator
        );
    });
}

#[test]
fn edit_cycle_for_proposed_markets() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            2..4,
            ScoringRule::AmmCdaHybrid,
        );

        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        let edit_reason = vec![0_u8; <Runtime as Config>::MaxEditReasonLen::get() as usize];

        // Now it should work for the designated origin.
        assert_ok!(PredictionMarkets::request_edit(
            RuntimeOrigin::signed(RequestEditOrigin::get()),
            0,
            edit_reason
        ));

        assert!(MarketIdsForEdit::<Runtime>::contains_key(0));

        // BOB was the oracle before through simple_create_categorical_market
        // After this edit its changed to ALICE
        assert_ok!(PredictionMarkets::edit_market(
            RuntimeOrigin::signed(ALICE),
            Asset::Ztg,
            0,
            CHARLIE,
            MarketPeriod::Block(2..4),
            get_deadlines(),
            gen_metadata(2),
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::AmmCdaHybrid
        ));
        let edited_market = MarketCommons::market(&0).expect("Market not found");
        System::assert_last_event(Event::MarketEdited(0, edited_market).into());
        assert!(!MarketIdsForEdit::<Runtime>::contains_key(0));
        // verify oracle is CHARLIE
        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().oracle, CHARLIE);
    });
}

#[cfg(feature = "parachain")]
#[test]
fn edit_market_with_foreign_asset() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            0..2,
            ScoringRule::AmmCdaHybrid,
        );

        // make sure it's in status proposed
        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        let edit_reason = vec![0_u8; <Runtime as Config>::MaxEditReasonLen::get() as usize];

        // Now it should work for the designated origin.
        assert_ok!(PredictionMarkets::request_edit(
            RuntimeOrigin::signed(RequestEditOrigin::get()),
            0,
            edit_reason
        ));

        assert!(MarketIdsForEdit::<Runtime>::contains_key(0));

        // ALICE is market creator through simple_create_categorical_market
        // As per Mock asset_registry genesis ForeignAsset(50) is not registered in asset_registry.
        assert_noop!(
            PredictionMarkets::edit_market(
                RuntimeOrigin::signed(ALICE),
                Asset::ForeignAsset(50),
                0,
                CHARLIE,
                MarketPeriod::Block(0..2),
                get_deadlines(),
                gen_metadata(2),
                MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
                Some(MarketDisputeMechanism::SimpleDisputes),
                ScoringRule::AmmCdaHybrid
            ),
            Error::<Runtime>::UnregisteredForeignAsset
        );
        // As per Mock asset_registry genesis ForeignAsset(420) has allow_as_base_asset set to false.
        assert_noop!(
            PredictionMarkets::edit_market(
                RuntimeOrigin::signed(ALICE),
                Asset::ForeignAsset(420),
                0,
                CHARLIE,
                MarketPeriod::Block(0..2),
                get_deadlines(),
                gen_metadata(2),
                MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
                Some(MarketDisputeMechanism::SimpleDisputes),
                ScoringRule::AmmCdaHybrid
            ),
            Error::<Runtime>::InvalidBaseAsset,
        );
        // As per Mock asset_registry genesis ForeignAsset(100) has allow_as_base_asset set to true.
        assert_ok!(PredictionMarkets::edit_market(
            RuntimeOrigin::signed(ALICE),
            Asset::ForeignAsset(100),
            0,
            CHARLIE,
            MarketPeriod::Block(0..2),
            get_deadlines(),
            gen_metadata(2),
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::AmmCdaHybrid
        ));
        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.base_asset, Asset::ForeignAsset(100));
    });
}
