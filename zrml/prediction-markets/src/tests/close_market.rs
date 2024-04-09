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
use test_case::test_case;

use crate::MarketIdsPerCloseBlock;
use sp_runtime::traits::Zero;

// TODO(#1239) Split test
#[test]
fn close_trusted_market_works() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 10;
        let market_creator = ALICE;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(market_creator),
            BaseAsset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            Deadlines {
                grace_period: 0,
                oracle_duration: <Runtime as Config>::MinOracleDuration::get(),
                dispute_duration: Zero::zero(),
            },
            gen_metadata(0x99),
            MarketCreation::Permissionless,
            MarketType::Categorical(3),
            None,
            ScoringRule::AmmCdaHybrid,
        ));

        let market_id = 0;
        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.dispute_mechanism, None);

        let new_end = end / 2;
        assert_ne!(new_end, end);
        run_to_block(new_end);

        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.status, MarketStatus::Active);

        let auto_closes = MarketIdsPerCloseBlock::<Runtime>::get(end);
        assert_eq!(auto_closes.first().cloned().unwrap(), market_id);

        assert_ok!(PredictionMarkets::close_trusted_market(
            RuntimeOrigin::signed(market_creator),
            market_id
        ));
        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.period, MarketPeriod::Block(0..new_end));
        assert_eq!(market.status, MarketStatus::Closed);

        let auto_closes = MarketIdsPerCloseBlock::<Runtime>::get(end);
        assert_eq!(auto_closes.len(), 0);
    });
}

#[test]
fn fails_if_caller_is_not_market_creator() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 10;
        let market_creator = ALICE;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(market_creator),
            BaseAsset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            Deadlines {
                grace_period: 0,
                oracle_duration: <Runtime as Config>::MinOracleDuration::get(),
                dispute_duration: Zero::zero(),
            },
            gen_metadata(0x99),
            MarketCreation::Permissionless,
            MarketType::Categorical(3),
            None,
            ScoringRule::AmmCdaHybrid,
        ));
        run_to_block(end - 1);
        assert_noop!(
            PredictionMarkets::close_trusted_market(RuntimeOrigin::signed(BOB), 0),
            Error::<Runtime>::CallerNotMarketCreator
        );
    });
}

#[test]
fn close_trusted_market_fails_if_not_trusted() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 10;
        let market_creator = ALICE;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(market_creator),
            BaseAsset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            Deadlines {
                grace_period: 0,
                oracle_duration: <Runtime as Config>::MinOracleDuration::get(),
                dispute_duration: <Runtime as Config>::MinDisputeDuration::get(),
            },
            gen_metadata(0x99),
            MarketCreation::Permissionless,
            MarketType::Categorical(3),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::AmmCdaHybrid,
        ));

        let market_id = 0;
        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.dispute_mechanism, Some(MarketDisputeMechanism::Court));

        run_to_block(end / 2);

        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.status, MarketStatus::Active);

        assert_noop!(
            PredictionMarkets::close_trusted_market(
                RuntimeOrigin::signed(market_creator),
                market_id
            ),
            Error::<Runtime>::MarketIsNotTrusted
        );
    });
}

#[test_case(MarketStatus::Closed; "closed")]
#[test_case(MarketStatus::Proposed; "proposed")]
#[test_case(MarketStatus::Resolved; "resolved")]
#[test_case(MarketStatus::Disputed; "disputed")]
#[test_case(MarketStatus::Reported; "report")]
fn close_trusted_market_fails_if_invalid_market_state(status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        let end = 10;
        let market_creator = ALICE;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(market_creator),
            BaseAsset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            Deadlines {
                grace_period: 0,
                oracle_duration: <Runtime as Config>::MinOracleDuration::get(),
                dispute_duration: Zero::zero(),
            },
            gen_metadata(0x99),
            MarketCreation::Permissionless,
            MarketType::Categorical(3),
            None,
            ScoringRule::AmmCdaHybrid,
        ));

        let market_id = 0;
        assert_ok!(MarketCommons::mutate_market(&market_id, |market| {
            market.status = status;
            Ok(())
        }));

        assert_noop!(
            PredictionMarkets::close_trusted_market(
                RuntimeOrigin::signed(market_creator),
                market_id
            ),
            Error::<Runtime>::MarketIsNotActive
        );
    });
}

#[test]
fn fails_if_market_is_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PredictionMarkets::close_trusted_market(RuntimeOrigin::signed(ALICE), 3),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist
        );
    });
}

#[test]
fn does_trigger_market_transition_api_permissionless() {
    ExtBuilder::default().build().execute_with(|| {
        StateTransitionMock::ensure_empty_state();
        simple_create_categorical_market(
            BaseAsset::Ztg,
            MarketCreation::Permissionless,
            1..2,
            ScoringRule::AmmCdaHybrid,
        );
        assert_ok!(PredictionMarkets::close_market(&0));
        assert!(StateTransitionMock::on_closure_triggered());
    });
}
