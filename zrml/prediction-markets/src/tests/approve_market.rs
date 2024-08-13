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

use crate::MarketIdsForEdit;
use sp_runtime::DispatchError;

#[test_case(MarketStatus::Active)]
#[test_case(MarketStatus::Closed)]
#[test_case(MarketStatus::Reported)]
#[test_case(MarketStatus::Disputed)]
#[test_case(MarketStatus::Resolved)]
fn fails_if_market_status_is_not_proposed(market_status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            0..2,
            ScoringRule::AmmCdaHybrid,
        );
        let market_id = 0;
        assert_ok!(MarketCommons::mutate_market(&market_id, |market| {
            market.status = market_status;
            Ok(())
        }));
        assert_noop!(
            PredictionMarkets::approve_market(
                RuntimeOrigin::signed(ApproveOrigin::get()),
                market_id
            ),
            Error::<Runtime>::MarketIsNotProposed
        );
    });
}

#[test]
fn it_allows_advisory_origin_to_approve_markets() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            0..2,
            ScoringRule::AmmCdaHybrid,
        );

        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        // Make sure it fails for the random joe
        assert_noop!(
            PredictionMarkets::approve_market(RuntimeOrigin::signed(BOB), 0),
            DispatchError::BadOrigin
        );

        assert_ok!(PredictionMarkets::approve_market(
            RuntimeOrigin::signed(ApproveOrigin::get()),
            0
        ));

        let after_market = MarketCommons::market(&0);
        assert_eq!(after_market.unwrap().status, MarketStatus::Active);
    });
}

#[test]
fn market_with_edit_request_cannot_be_approved() {
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

        assert_ok!(PredictionMarkets::request_edit(
            RuntimeOrigin::signed(RequestEditOrigin::get()),
            0,
            edit_reason
        ));

        assert!(MarketIdsForEdit::<Runtime>::contains_key(0));
        assert_noop!(
            PredictionMarkets::approve_market(RuntimeOrigin::signed(ApproveOrigin::get()), 0),
            Error::<Runtime>::MarketEditRequestAlreadyInProgress
        );
    });
}

#[test]
fn approve_market_correctly_unreserves_advisory_bond() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: AssetOf<Runtime>| {
        reserve_sentinel_amounts();
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..100),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::AmmCdaHybrid,
        ));
        let market_id = 0;
        let alice_balance_before = Balances::free_balance(ALICE);
        check_reserve(&ALICE, AdvisoryBond::get() + OracleBond::get());
        assert_ok!(PredictionMarkets::approve_market(
            RuntimeOrigin::signed(ApproveOrigin::get()),
            market_id
        ));
        check_reserve(&ALICE, OracleBond::get());
        assert_eq!(Balances::free_balance(ALICE), alice_balance_before + AdvisoryBond::get());
        let market = MarketCommons::market(&market_id).unwrap();
        assert!(market.bonds.creation.unwrap().is_settled);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}
