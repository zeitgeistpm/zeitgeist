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

use crate::{MarketIdsPerDisputeBlock, MarketIdsPerReportBlock};
use sp_runtime::traits::Zero;
use zeitgeist_primitives::types::{Bond, OutcomeReport, Report};

#[test]
fn it_correctly_resolves_a_market_that_was_reported_on() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 2;
        simple_create_categorical_market(
            BaseAsset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::Lmsr,
        );

        assert_ok!(PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(CHARLIE), 0, CENT));

        let market = MarketCommons::market(&0).unwrap();
        let report_at = end + market.deadlines.grace_period + 1;
        run_to_block(report_at);

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));

        let reported_ids =
            MarketIdsPerReportBlock::<Runtime>::get(report_at + market.deadlines.dispute_duration);
        assert_eq!(reported_ids.len(), 1);
        let id = reported_ids[0];
        assert_eq!(id, 0);

        run_blocks(market.deadlines.dispute_duration);

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Resolved);

        // Check balance of winning outcome asset.
        let share_b = Asset::CategoricalOutcome(0, 1);
        let share_b_total = AssetManager::total_issuance(share_b);
        assert_eq!(share_b_total, CENT);
        let share_b_bal = AssetManager::free_balance(share_b, &CHARLIE);
        assert_eq!(share_b_bal, CENT);

        let share_a = Asset::CategoricalOutcome(0, 0);
        let share_a_total = AssetManager::total_issuance(share_a);
        assert_eq!(share_a_total, 0);
        let share_a_bal = AssetManager::free_balance(share_a, &CHARLIE);
        assert_eq!(share_a_bal, 0);

        let share_c = Asset::CategoricalOutcome(0, 2);
        let share_c_total = AssetManager::total_issuance(share_c);
        assert_eq!(share_c_total, 0);
        let share_c_bal = AssetManager::free_balance(share_c, &CHARLIE);
        assert_eq!(share_c_bal, 0);

        assert!(market.bonds.creation.unwrap().is_settled);
        assert!(market.bonds.oracle.unwrap().is_settled);
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_approved_advised_market_on_oracle_report()
 {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: BaseAsset| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        assert_ok!(PredictionMarkets::approve_market(
            RuntimeOrigin::signed(ApproveOrigin::get()),
            0
        ));
        let alice_balance_before = Balances::free_balance(ALICE);
        check_reserve(&ALICE, OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        let report_at = grace_period + 1;
        run_to_block(report_at);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // Check that nothing got slashed
        assert_eq!(Balances::free_balance(ALICE), alice_balance_before + OracleBond::get());
    };
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::CampaignAsset(100));
    });
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_approved_advised_market_on_outsider_report()
 {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: BaseAsset| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        assert_ok!(PredictionMarkets::approve_market(
            RuntimeOrigin::signed(ApproveOrigin::get()),
            0
        ));
        let alice_balance_before = Balances::free_balance(ALICE);
        check_reserve(&ALICE, OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        let report_at = grace_period + market.deadlines.oracle_duration + 1;
        run_to_block(report_at);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_blocks(market.deadlines.dispute_duration);
        // Check that oracle bond got slashed
        check_reserve(&ALICE, 0);
        assert_eq!(Balances::free_balance(ALICE), alice_balance_before);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::CampaignAsset(100));
    });
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_permissionless_market_with_correct_disputed_outcome_with_oracle_report()
 {
    // Oracle reports in time but incorrect report, so OracleBond gets slashed on resolution
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: BaseAsset| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        let alice_balance_before = Balances::free_balance(ALICE);
        check_reserve(&ALICE, ValidityBond::get() + OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));
        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), 0,));
        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // ValidityBond bond is returned but OracleBond is slashed
        assert_eq!(Balances::free_balance(ALICE), alice_balance_before + ValidityBond::get());
    };
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::CampaignAsset(100));
    });
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_approved_advised_market_with_correct_disputed_outcome_with_oracle_report()
 {
    // Oracle reports in time but incorrect report, so OracleBond gets slashed on resolution
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: BaseAsset| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        assert_ok!(PredictionMarkets::approve_market(
            RuntimeOrigin::signed(ApproveOrigin::get()),
            0
        ));
        let alice_balance_before = Balances::free_balance(ALICE);
        check_reserve(&ALICE, OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));
        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), 0,));
        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // ValidityBond bond is returned but OracleBond is slashed
        assert_eq!(Balances::free_balance(ALICE), alice_balance_before);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::CampaignAsset(100));
    });
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_permissionless_market_with_wrong_disputed_outcome_with_oracle_report()
 {
    // Oracle reports in time and correct report, so OracleBond does not get slashed on resolution
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: BaseAsset| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        let alice_balance_before = Balances::free_balance(ALICE);
        check_reserve(&ALICE, ValidityBond::get() + OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));
        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(EVE), 0,));
        // EVE disputes with wrong outcome
        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(EVE),
            0,
            OutcomeReport::Categorical(1)
        ));
        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(0)
        ));
        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // ValidityBond bond is returned but OracleBond is not slashed
        assert_eq!(
            Balances::free_balance(ALICE),
            alice_balance_before + ValidityBond::get() + OracleBond::get()
        );
    };
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::CampaignAsset(100));
    });
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_advised_approved_market_with_wrong_disputed_outcome_with_oracle_report()
 {
    // Oracle reports in time and correct report, so OracleBond does not get slashed on resolution
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: BaseAsset| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        assert_ok!(PredictionMarkets::approve_market(
            RuntimeOrigin::signed(ApproveOrigin::get()),
            0
        ));
        let alice_balance_before = Balances::free_balance(ALICE);
        check_reserve(&ALICE, OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));
        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(EVE), 0,));
        // EVE disputes with wrong outcome
        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(EVE),
            0,
            OutcomeReport::Categorical(1)
        ));
        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(0)
        ));
        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // ValidityBond bond is returned but OracleBond is not slashed
        assert_eq!(Balances::free_balance(ALICE), alice_balance_before + OracleBond::get());
    };
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::CampaignAsset(100));
    });
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_permissionless_market_with_disputed_outcome_with_outsider_report()
 {
    // Oracle does not report in time, so OracleBond gets slashed on resolution
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: BaseAsset| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));

        let alice_balance_before = Balances::free_balance(ALICE);
        check_reserve(&ALICE, ValidityBond::get() + OracleBond::get());

        let outsider = CHARLIE;

        let market = MarketCommons::market(&0).unwrap();
        let after_oracle_duration =
            end + market.deadlines.grace_period + market.deadlines.oracle_duration + 1;
        run_to_block(after_oracle_duration);
        // CHARLIE is not an Oracle
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(outsider),
            0,
            OutcomeReport::Categorical(0)
        ));
        let outsider_balance_before = Balances::free_balance(outsider);
        check_reserve(&outsider, <Runtime as Config>::OutsiderBond::get());

        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(EVE), 0,));
        // EVE disputes with wrong outcome
        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(EVE),
            0,
            OutcomeReport::Categorical(1)
        ));
        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(FRED),
            0,
            OutcomeReport::Categorical(0)
        ));
        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // ValidityBond bond is returned but OracleBond is slashed
        assert_eq!(Balances::free_balance(ALICE), alice_balance_before + ValidityBond::get());

        check_reserve(&outsider, 0);
        assert_eq!(
            Balances::free_balance(outsider),
            outsider_balance_before + OracleBond::get() + <Runtime as Config>::OutsiderBond::get()
        );
    };
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::CampaignAsset(100));
    });
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_advised_approved_market_with_disputed_outcome_with_outsider_report()
 {
    // Oracle does not report in time, so OracleBond gets slashed on resolution
    // NOTE: Bonds are always in ZTG
    let test = |base_asset: BaseAsset| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));

        let outsider = CHARLIE;

        assert_ok!(PredictionMarkets::approve_market(
            RuntimeOrigin::signed(ApproveOrigin::get()),
            0
        ));
        let alice_balance_before = Balances::free_balance(ALICE);
        check_reserve(&ALICE, OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let after_oracle_duration =
            end + market.deadlines.grace_period + market.deadlines.oracle_duration + 1;
        run_to_block(after_oracle_duration);
        // CHARLIE is not an Oracle
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(outsider),
            0,
            OutcomeReport::Categorical(0)
        ));
        let outsider_balance_before = Balances::free_balance(outsider);
        check_reserve(&outsider, <Runtime as Config>::OutsiderBond::get());

        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(EVE), 0,));
        // EVE disputes with wrong outcome
        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(EVE),
            0,
            OutcomeReport::Categorical(1)
        ));
        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(FRED),
            0,
            OutcomeReport::Categorical(0)
        ));
        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // ValidityBond bond is returned but OracleBond is slashed
        assert_eq!(Balances::free_balance(ALICE), alice_balance_before);

        check_reserve(&outsider, 0);
        assert_eq!(
            Balances::free_balance(outsider),
            outsider_balance_before + OracleBond::get() + <Runtime as Config>::OutsiderBond::get()
        );
    };
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::CampaignAsset(100));
    });
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::ForeignAsset(100));
    });
}

#[test]
fn trusted_market_complete_lifecycle() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 3;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
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
            ScoringRule::Lmsr,
        ));
        let market_id = 0;
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(FRED),
            market_id,
            BASE
        ));
        run_to_block(end);
        let outcome = OutcomeReport::Categorical(1);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            market_id,
            outcome.clone()
        ));
        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.status, MarketStatus::Resolved);
        assert_eq!(market.report, Some(Report { at: end, by: BOB, outcome: outcome.clone() }));
        assert_eq!(market.resolved_outcome, Some(outcome));
        assert_eq!(market.dispute_mechanism, None);
        assert!(market.bonds.oracle.unwrap().is_settled);
        assert_eq!(market.bonds.outsider, None);
        assert_eq!(market.bonds.dispute, None);
        assert_ok!(PredictionMarkets::redeem_shares(RuntimeOrigin::signed(FRED), market_id));
        // Ensure that we don't accidentally leave any artifacts.
        assert!(MarketIdsPerDisputeBlock::<Runtime>::iter().next().is_none());
        assert!(MarketIdsPerReportBlock::<Runtime>::iter().next().is_none());
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_permissionless_market_on_oracle_report()
 {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: BaseAsset| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        let alice_balance_before = Balances::free_balance(ALICE);
        check_reserve(&ALICE, ValidityBond::get() + OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));
        run_to_block(grace_period + market.deadlines.dispute_duration + 1);
        check_reserve(&ALICE, 0);
        assert_eq!(
            Balances::free_balance(ALICE),
            alice_balance_before + ValidityBond::get() + OracleBond::get()
        );
    };
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::CampaignAsset(100));
    });
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_permissionless_market_on_outsider_report()
 {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: BaseAsset| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..100),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        let alice_balance_before = Balances::free_balance(ALICE);
        check_reserve(&ALICE, ValidityBond::get() + OracleBond::get());

        let charlie_balance_before = Balances::free_balance(CHARLIE);
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        let report_at = grace_period + market.deadlines.oracle_duration + 1;
        run_to_block(report_at);

        assert!(market.bonds.outsider.is_none());
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(1)
        ));

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(
            market.bonds.outsider,
            Some(Bond::new(CHARLIE, <Runtime as Config>::OutsiderBond::get()))
        );
        check_reserve(&CHARLIE, <Runtime as Config>::OutsiderBond::get());
        assert_eq!(
            Balances::free_balance(CHARLIE),
            charlie_balance_before - <Runtime as Config>::OutsiderBond::get()
        );
        let charlie_balance_before = Balances::free_balance(CHARLIE);

        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // Check that validity bond didn't get slashed, but oracle bond did
        assert_eq!(Balances::free_balance(ALICE), alice_balance_before + ValidityBond::get());

        check_reserve(&CHARLIE, 0);
        // Check that the outsider gets the OracleBond together with the OutsiderBond
        assert_eq!(
            Balances::free_balance(CHARLIE),
            charlie_balance_before + OracleBond::get() + <Runtime as Config>::OutsiderBond::get()
        );
        let market = MarketCommons::market(&0).unwrap();
        assert!(market.bonds.outsider.unwrap().is_settled);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::CampaignAsset(100));
    });
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::ForeignAsset(100));
    });
}

#[test]
fn does_trigger_market_transition_api() {
    ExtBuilder::default().build().execute_with(|| {
        StateTransitionMock::ensure_empty_state();
        let end = 3;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
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
            ScoringRule::Lmsr,
        ));
        run_to_block(end);
        let outcome = OutcomeReport::Categorical(1);
        assert_ok!(PredictionMarkets::report(RuntimeOrigin::signed(BOB), 0, outcome.clone()));
        assert!(StateTransitionMock::on_resolution_triggered());
    });
}
