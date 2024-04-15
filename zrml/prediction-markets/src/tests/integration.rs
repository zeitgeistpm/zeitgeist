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
use crate::MarketIdsPerDisputeBlock;
use alloc::collections::BTreeMap;
use orml_traits::MultiReservableCurrency;
use sp_runtime::Perquintill;
use zeitgeist_primitives::{
    constants::MILLISECS_PER_BLOCK,
    types::{OutcomeReport, ScalarPosition},
};
use zrml_court::types::{CourtStatus, Draw, Vote};
use zrml_global_disputes::{
    types::{OutcomeInfo, Possession},
    GlobalDisputesPalletApi, Outcomes, PossessionOf,
};

#[test]
fn it_appeals_a_court_market_to_global_dispute() {
    let test = |base_asset: BaseAsset| {
        let mut free_before = BTreeMap::new();
        let jurors =
            1000..(1000 + <Runtime as zrml_court::Config>::MaxSelectedDraws::get() as u128);
        for j in jurors {
            let amount = <Runtime as zrml_court::Config>::MinJurorStake::get() + j;
            assert_ok!(AssetManager::deposit(Asset::Ztg, &j, amount + SENTINEL_AMOUNT));
            assert_ok!(Court::join_court(RuntimeOrigin::signed(j), amount));
            free_before.insert(j, Balances::free_balance(j));
        }

        let end = 2;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::AmmCdaHybrid,
        ));

        let market_id = 0;
        let market = MarketCommons::market(&0).unwrap();

        let report_at = end + market.deadlines.grace_period + 1;
        run_to_block(report_at);

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            market_id,
            OutcomeReport::Categorical(0)
        ));

        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), market_id,));

        for _ in 0..(<Runtime as zrml_court::Config>::MaxAppeals::get() - 1) {
            simulate_appeal_cycle(market_id);
            assert_ok!(Court::appeal(RuntimeOrigin::signed(BOB), market_id));
        }

        let court = zrml_court::Courts::<Runtime>::get(market_id).unwrap();
        let appeals = court.appeals;
        assert_eq!(
            appeals.len(),
            (<Runtime as zrml_court::Config>::MaxAppeals::get() - 1) as usize
        );

        assert_noop!(
            PredictionMarkets::start_global_dispute(RuntimeOrigin::signed(BOB), market_id),
            Error::<Runtime>::MarketDisputeMechanismNotFailed
        );

        simulate_appeal_cycle(market_id);
        assert_ok!(Court::appeal(RuntimeOrigin::signed(BOB), market_id));

        assert_noop!(
            Court::appeal(RuntimeOrigin::signed(BOB), market_id),
            zrml_court::Error::<Runtime>::MaxAppealsReached
        );

        assert!(!GlobalDisputes::does_exist(&market_id));

        assert_ok!(PredictionMarkets::start_global_dispute(RuntimeOrigin::signed(BOB), market_id));

        let now = <frame_system::Pallet<Runtime>>::block_number();

        assert!(GlobalDisputes::does_exist(&market_id));
        System::assert_last_event(Event::GlobalDisputeStarted(market_id).into());

        // report check
        let possession: PossessionOf<Runtime> =
            Possession::Shared { owners: frame_support::BoundedVec::try_from(vec![BOB]).unwrap() };
        let outcome_info = OutcomeInfo { outcome_sum: Zero::zero(), possession };
        assert_eq!(
            Outcomes::<Runtime>::get(market_id, &OutcomeReport::Categorical(0)).unwrap(),
            outcome_info
        );

        let add_outcome_end = now + GlobalDisputes::get_add_outcome_period();
        let vote_end = add_outcome_end + GlobalDisputes::get_vote_period();
        let market_ids = MarketIdsPerDisputeBlock::<Runtime>::get(vote_end);
        assert_eq!(market_ids, vec![market_id]);
        assert!(GlobalDisputes::is_active(&market_id));

        assert_noop!(
            PredictionMarkets::start_global_dispute(RuntimeOrigin::signed(CHARLIE), market_id),
            Error::<Runtime>::GlobalDisputeExistsAlready
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
fn the_entire_market_lifecycle_works_with_timestamps() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::AmmCdaHybrid
        ));

        // is ok
        assert_ok!(PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(BOB), 0, CENT));
        let market = MarketCommons::market(&0).unwrap();

        // set the timestamp
        set_timestamp_for_on_initialize(100_000_000);
        run_to_block(2); // Trigger `on_initialize`; must be at least block #2.
        let grace_period: u64 = market.deadlines.grace_period * MILLISECS_PER_BLOCK as u64;
        Timestamp::set_timestamp(100_000_000 + grace_period);

        assert_noop!(
            PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(BOB), 0, CENT),
            Error::<Runtime>::MarketIsNotActive,
        );

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
    });
}

#[test]
fn full_scalar_market_lifecycle() {
    let test = |base_asset: BaseAsset| {
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            get_deadlines(),
            gen_metadata(3),
            MarketCreation::Permissionless,
            MarketType::Scalar(10..=30),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::AmmCdaHybrid
        ));

        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(CHARLIE),
            0,
            100 * BASE
        ));

        // check balances
        let market = &MarketCommons::market(&0).unwrap();
        let assets = market.outcome_assets();
        assert_eq!(assets.len(), 2);
        for asset in assets.iter() {
            let bal = AssetManager::free_balance((*asset).into(), &CHARLIE);
            assert_eq!(bal, 100 * BASE);
        }

        set_timestamp_for_on_initialize(100_000_000);
        let report_at = 2;
        run_to_block(report_at); // Trigger `on_initialize`; must be at least block #2.
        let grace_period: u64 = market.deadlines.grace_period * MILLISECS_PER_BLOCK as u64;
        Timestamp::set_timestamp(100_000_000 + grace_period);

        // report
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Scalar(100)
        ));

        let market_after_report = MarketCommons::market(&0).unwrap();
        assert!(market_after_report.report.is_some());
        let report = market_after_report.report.unwrap();
        assert_eq!(report.at, report_at);
        assert_eq!(report.by, BOB);
        assert_eq!(report.outcome, OutcomeReport::Scalar(100));

        // dispute
        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(DAVE), 0));
        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(DAVE),
            0,
            OutcomeReport::Scalar(25)
        ));
        let disputes = zrml_simple_disputes::Disputes::<Runtime>::get(0);
        assert_eq!(disputes.len(), 1);

        run_blocks(market.deadlines.dispute_duration);

        let market_after_resolve = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after_resolve.status, MarketStatus::Resolved);
        let disputes = zrml_simple_disputes::Disputes::<Runtime>::get(0);
        assert_eq!(disputes.len(), 0);

        // give EVE some shares
        assert_ok!(AssetManager::transfer(
            RuntimeOrigin::signed(CHARLIE),
            EVE,
            Asset::ScalarOutcome(0, ScalarPosition::Short),
            50 * BASE
        ));

        assert_eq!(
            AssetManager::free_balance(Asset::ScalarOutcome(0, ScalarPosition::Short), &CHARLIE),
            50 * BASE
        );

        assert_ok!(PredictionMarkets::redeem_shares(RuntimeOrigin::signed(CHARLIE), 0));
        for asset in assets.iter() {
            let bal = AssetManager::free_balance((*asset).into(), &CHARLIE);
            assert_eq!(bal, 0);
        }

        // check payouts is right for each CHARLIE and EVE
        let base_asset_bal_charlie = AssetManager::free_balance(base_asset.into(), &CHARLIE);
        let base_asset_bal_eve = AssetManager::free_balance(base_asset.into(), &EVE);
        assert_eq!(base_asset_bal_charlie, 98750 * CENT); // 75 (LONG) + 12.5 (SHORT) + 900 (balance)
        assert_eq!(base_asset_bal_eve, 1000 * BASE);
        System::assert_has_event(
            Event::TokensRedeemed(
                0,
                Asset::ScalarOutcome(0, ScalarPosition::Long),
                100 * BASE,
                75 * BASE,
                CHARLIE,
            )
            .into(),
        );
        System::assert_has_event(
            Event::TokensRedeemed(
                0,
                Asset::ScalarOutcome(0, ScalarPosition::Short),
                50 * BASE,
                1250 * CENT, // 12.5
                CHARLIE,
            )
            .into(),
        );

        assert_ok!(PredictionMarkets::redeem_shares(RuntimeOrigin::signed(EVE), 0));
        let base_asset_bal_eve_after = AssetManager::free_balance(base_asset.into(), &EVE);
        assert_eq!(base_asset_bal_eve_after, 101250 * CENT); // 12.5 (SHORT) + 1000 (balance)
        System::assert_last_event(
            Event::TokensRedeemed(
                0,
                Asset::ScalarOutcome(0, ScalarPosition::Short),
                50 * BASE,
                1250 * CENT, // 12.5
                EVE,
            )
            .into(),
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
fn authorized_correctly_resolves_disputed_market() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: BaseAsset| {
        let end = 2;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Authorized),
            ScoringRule::AmmCdaHybrid,
        ));
        assert_ok!(PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(CHARLIE), 0, CENT));

        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));

        let charlie_balance = AssetManager::free_balance(base_asset.into(), &CHARLIE);
        assert_eq!(charlie_balance, 1_000 * BASE - CENT);

        let dispute_at = grace_period + 1 + 1;
        run_to_block(dispute_at);
        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), 0,));

        if base_asset == BaseAsset::Ztg {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(
                charlie_balance,
                1_000 * BASE - CENT - <Runtime as Config>::DisputeBond::get()
            );
        } else {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - <Runtime as Config>::DisputeBond::get());
            let charlie_balance = AssetManager::free_balance(base_asset.into(), &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - CENT);
        }

        // Fred authorizses an outcome, but fat-fingers it on the first try.
        assert_ok!(Authorized::authorize_market_outcome(
            RuntimeOrigin::signed(AuthorizedDisputeResolutionUser::get()),
            0,
            OutcomeReport::Categorical(0)
        ));
        assert_ok!(Authorized::authorize_market_outcome(
            RuntimeOrigin::signed(AuthorizedDisputeResolutionUser::get()),
            0,
            OutcomeReport::Categorical(1)
        ));

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Disputed);

        // check everyone's deposits
        let charlie_reserved = Balances::reserved_balance(CHARLIE);
        assert_eq!(charlie_reserved, <Runtime as Config>::DisputeBond::get());

        let market_ids_1 = MarketIdsPerDisputeBlock::<Runtime>::get(
            dispute_at + <Runtime as zrml_authorized::Config>::CorrectionPeriod::get(),
        );
        assert_eq!(market_ids_1.len(), 1);

        if base_asset == BaseAsset::Ztg {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(
                charlie_balance,
                1_000 * BASE - CENT - <Runtime as Config>::DisputeBond::get()
            );
        } else {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - <Runtime as Config>::DisputeBond::get());
            let charlie_balance = AssetManager::free_balance(base_asset.into(), &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - CENT);
        }

        run_blocks(<Runtime as zrml_authorized::Config>::CorrectionPeriod::get() - 1);

        let market_after = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after.status, MarketStatus::Disputed);

        if base_asset == BaseAsset::Ztg {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(
                charlie_balance,
                1_000 * BASE - CENT - <Runtime as Config>::DisputeBond::get()
            );
        } else {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - <Runtime as Config>::DisputeBond::get());
            let charlie_balance = AssetManager::free_balance(base_asset.into(), &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - CENT);
        }

        run_blocks(1);

        if base_asset == BaseAsset::Ztg {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(
                charlie_balance,
                1_000 * BASE - CENT + <Runtime as Config>::OracleBond::get()
            );
        } else {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE + <Runtime as Config>::OracleBond::get());
            let charlie_balance = AssetManager::free_balance(base_asset.into(), &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - CENT);
        }

        let market_after = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after.status, MarketStatus::Resolved);
        let disputes = zrml_simple_disputes::Disputes::<Runtime>::get(0);
        assert_eq!(disputes.len(), 0);

        assert_ok!(PredictionMarkets::redeem_shares(RuntimeOrigin::signed(CHARLIE), 0));

        if base_asset == BaseAsset::Ztg {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE + <Runtime as Config>::OracleBond::get());
        } else {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE + <Runtime as Config>::OracleBond::get());
            let charlie_balance = AssetManager::free_balance(base_asset.into(), &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE);
        }
        let charlie_reserved_2 = AssetManager::reserved_balance(Asset::Ztg, &CHARLIE);
        assert_eq!(charlie_reserved_2, 0);

        let alice_balance = AssetManager::free_balance(Asset::Ztg, &ALICE);
        assert_eq!(alice_balance, 1_000 * BASE - <Runtime as Config>::OracleBond::get());

        // bob kinda gets away scot-free since Alice is held responsible
        // for her designated reporter
        let bob_balance = AssetManager::free_balance(Asset::Ztg, &BOB);
        assert_eq!(bob_balance, 1_000 * BASE);

        assert!(market_after.bonds.creation.unwrap().is_settled);
        assert!(market_after.bonds.oracle.unwrap().is_settled);
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
fn it_resolves_a_disputed_court_market() {
    let test = |base_asset: BaseAsset| {
        let juror_0 = 1000;
        let juror_1 = 1001;
        let juror_2 = 1002;
        let juror_3 = 1003;
        let juror_4 = 1004;
        let juror_5 = 1005;

        for j in &[juror_0, juror_1, juror_2, juror_3, juror_4, juror_5] {
            let amount = <Runtime as zrml_court::Config>::MinJurorStake::get() + *j;
            assert_ok!(AssetManager::deposit(Asset::Ztg, j, amount + SENTINEL_AMOUNT));
            assert_ok!(Court::join_court(RuntimeOrigin::signed(*j), amount));
        }

        // just to have enough jurors for the dispute
        for j in 1006..(1006 + Court::necessary_draws_weight(0usize) as u32) {
            let juror = j as u128;
            let amount = <Runtime as zrml_court::Config>::MinJurorStake::get() + juror;
            assert_ok!(AssetManager::deposit(Asset::Ztg, &juror, amount + SENTINEL_AMOUNT));
            assert_ok!(Court::join_court(RuntimeOrigin::signed(juror), amount));
        }

        let end = 2;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::AmmCdaHybrid,
        ));

        let market_id = 0;
        let market = MarketCommons::market(&0).unwrap();

        let report_at = end + market.deadlines.grace_period + 1;
        run_to_block(report_at);

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            market_id,
            OutcomeReport::Categorical(0)
        ));

        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), market_id,));

        let court = zrml_court::Courts::<Runtime>::get(market_id).unwrap();
        let vote_start = court.round_ends.pre_vote + 1;

        run_to_block(vote_start);

        // overwrite draws to disregard randomness
        zrml_court::SelectedDraws::<Runtime>::remove(market_id);
        let mut draws = zrml_court::SelectedDraws::<Runtime>::get(market_id);
        for juror in &[juror_0, juror_1, juror_2, juror_3, juror_4, juror_5] {
            let draw = Draw {
                court_participant: *juror,
                weight: 1,
                vote: Vote::Drawn,
                slashable: <Runtime as zrml_court::Config>::MinJurorStake::get(),
            };
            let index = draws
                .binary_search_by_key(juror, |draw| draw.court_participant)
                .unwrap_or_else(|j| j);
            draws.try_insert(index, draw).unwrap();
        }
        let old_draws = draws.clone();
        zrml_court::SelectedDraws::<Runtime>::insert(market_id, draws);

        let salt = <Runtime as frame_system::Config>::Hash::default();

        // outcome_0 is the plurality decision => right outcome
        let outcome_0 = OutcomeReport::Categorical(0);
        let vote_item_0 = VoteItem::Outcome(outcome_0.clone());
        // outcome_1 is the wrong outcome
        let outcome_1 = OutcomeReport::Categorical(1);
        let vote_item_1 = VoteItem::Outcome(outcome_1);

        let commitment_0 = BlakeTwo256::hash_of(&(juror_0, vote_item_0.clone(), salt));
        assert_ok!(Court::vote(RuntimeOrigin::signed(juror_0), market_id, commitment_0));

        // juror_1 votes for non-plurality outcome => slashed later
        let commitment_1 = BlakeTwo256::hash_of(&(juror_1, vote_item_1.clone(), salt));
        assert_ok!(Court::vote(RuntimeOrigin::signed(juror_1), market_id, commitment_1));

        let commitment_2 = BlakeTwo256::hash_of(&(juror_2, vote_item_0.clone(), salt));
        assert_ok!(Court::vote(RuntimeOrigin::signed(juror_2), market_id, commitment_2));

        let commitment_3 = BlakeTwo256::hash_of(&(juror_3, vote_item_0.clone(), salt));
        assert_ok!(Court::vote(RuntimeOrigin::signed(juror_3), market_id, commitment_3));

        // juror_4 fails to vote in time

        let commitment_5 = BlakeTwo256::hash_of(&(juror_5, vote_item_0.clone(), salt));
        assert_ok!(Court::vote(RuntimeOrigin::signed(juror_5), market_id, commitment_5));

        // juror_3 is denounced by juror_0 => slashed later
        assert_ok!(Court::denounce_vote(
            RuntimeOrigin::signed(juror_0),
            market_id,
            juror_3,
            vote_item_0.clone(),
            salt
        ));

        let aggregation_start = court.round_ends.vote + 1;
        run_to_block(aggregation_start);

        assert_ok!(Court::reveal_vote(
            RuntimeOrigin::signed(juror_0),
            market_id,
            vote_item_0.clone(),
            salt
        ));
        assert_ok!(Court::reveal_vote(
            RuntimeOrigin::signed(juror_1),
            market_id,
            vote_item_1,
            salt
        ));

        let wrong_salt = BlakeTwo256::hash_of(&69);
        assert_noop!(
            Court::reveal_vote(
                RuntimeOrigin::signed(juror_2),
                market_id,
                vote_item_0.clone(),
                wrong_salt
            ),
            zrml_court::Error::<Runtime>::CommitmentHashMismatch
        );
        assert_ok!(Court::reveal_vote(
            RuntimeOrigin::signed(juror_2),
            market_id,
            vote_item_0.clone(),
            salt
        ));

        assert_noop!(
            Court::reveal_vote(
                RuntimeOrigin::signed(juror_3),
                market_id,
                vote_item_0.clone(),
                salt
            ),
            zrml_court::Error::<Runtime>::VoteAlreadyDenounced
        );

        assert_noop!(
            Court::reveal_vote(
                RuntimeOrigin::signed(juror_4),
                market_id,
                vote_item_0.clone(),
                salt
            ),
            zrml_court::Error::<Runtime>::JurorDidNotVote
        );

        // juror_5 fails to reveal in time

        let resolve_at = court.round_ends.appeal;
        let market_ids = MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at);
        assert_eq!(market_ids.len(), 1);

        run_blocks(resolve_at);

        let market_after = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after.status, MarketStatus::Resolved);
        assert_eq!(market_after.resolved_outcome, Some(outcome_0));
        let court_after = zrml_court::Courts::<Runtime>::get(market_id).unwrap();
        assert_eq!(court_after.status, CourtStatus::Closed { winner: vote_item_0 });

        let free_juror_0_before = Balances::free_balance(juror_0);
        let free_juror_1_before = Balances::free_balance(juror_1);
        let free_juror_2_before = Balances::free_balance(juror_2);
        let free_juror_3_before = Balances::free_balance(juror_3);
        let free_juror_4_before = Balances::free_balance(juror_4);
        let free_juror_5_before = Balances::free_balance(juror_5);

        assert_ok!(Court::reassign_court_stakes(RuntimeOrigin::signed(juror_0), market_id));

        let free_juror_0_after = Balances::free_balance(juror_0);
        let slashable_juror_0 =
            old_draws.iter().find(|draw| draw.court_participant == juror_0).unwrap().slashable;
        let free_juror_1_after = Balances::free_balance(juror_1);
        let slashable_juror_1 =
            old_draws.iter().find(|draw| draw.court_participant == juror_1).unwrap().slashable;
        let free_juror_2_after = Balances::free_balance(juror_2);
        let slashable_juror_2 =
            old_draws.iter().find(|draw| draw.court_participant == juror_2).unwrap().slashable;
        let free_juror_3_after = Balances::free_balance(juror_3);
        let slashable_juror_3 =
            old_draws.iter().find(|draw| draw.court_participant == juror_3).unwrap().slashable;
        let free_juror_4_after = Balances::free_balance(juror_4);
        let slashable_juror_4 =
            old_draws.iter().find(|draw| draw.court_participant == juror_4).unwrap().slashable;
        let free_juror_5_after = Balances::free_balance(juror_5);
        let slashable_juror_5 =
            old_draws.iter().find(|draw| draw.court_participant == juror_5).unwrap().slashable;

        let mut total_slashed = 0;
        // juror_1 voted for the wrong outcome => slashed
        assert_eq!(free_juror_1_before - free_juror_1_after, slashable_juror_1);
        total_slashed += slashable_juror_1;
        // juror_3 was denounced by juror_0 => slashed
        assert_eq!(free_juror_3_before - free_juror_3_after, slashable_juror_3);
        total_slashed += slashable_juror_3;
        // juror_4 failed to vote => slashed
        assert_eq!(free_juror_4_before - free_juror_4_after, slashable_juror_4);
        total_slashed += slashable_juror_4;
        // juror_5 failed to reveal => slashed
        assert_eq!(free_juror_5_before - free_juror_5_after, slashable_juror_5);
        total_slashed += slashable_juror_5;
        // juror_0 and juror_2 voted for the right outcome => rewarded
        let total_winner_stake = slashable_juror_0 + slashable_juror_2;
        let juror_0_share = Perquintill::from_rational(slashable_juror_0, total_winner_stake);
        assert_eq!(free_juror_0_after, free_juror_0_before + juror_0_share * total_slashed);
        let juror_2_share = Perquintill::from_rational(slashable_juror_2, total_winner_stake);
        assert_eq!(free_juror_2_after, free_juror_2_before + juror_2_share * total_slashed);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::ForeignAsset(100));
    });
}

#[test]
fn outsider_reports_wrong_outcome() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: BaseAsset| {
        reserve_sentinel_amounts();

        let end = 100;
        let alice_balance_before = Balances::free_balance(ALICE);
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
            ScoringRule::AmmCdaHybrid,
        ));

        let outsider = CHARLIE;

        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        let report_at = grace_period + market.deadlines.oracle_duration + 1;
        run_to_block(report_at);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(outsider),
            0,
            OutcomeReport::Categorical(1)
        ));

        let outsider_balance_before = Balances::free_balance(outsider);
        check_reserve(&outsider, <Runtime as Config>::OutsiderBond::get());

        let dispute_at_0 = report_at + 1;
        run_to_block(dispute_at_0);
        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(EVE), 0,));
        check_reserve(&EVE, <Runtime as Config>::DisputeBond::get());

        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(DAVE),
            0,
            OutcomeReport::Categorical(0)
        ));

        let outcome_bond = zrml_simple_disputes::default_outcome_bond::<Runtime>(0);

        check_reserve(&DAVE, outcome_bond);

        let eve_balance_before = Balances::free_balance(EVE);
        let dave_balance_before = Balances::free_balance(DAVE);

        // on_resolution called
        run_blocks(market.deadlines.dispute_duration);

        assert_eq!(
            Balances::free_balance(ALICE),
            alice_balance_before - <Runtime as Config>::OracleBond::get()
        );

        check_reserve(&outsider, 0);
        assert_eq!(Balances::free_balance(outsider), outsider_balance_before);

        // disputor EVE gets the OracleBond and <Runtime as Config>::OutsiderBond and DisputeBond
        assert_eq!(
            Balances::free_balance(EVE),
            eve_balance_before
                + <Runtime as Config>::DisputeBond::get()
                + <Runtime as Config>::OutsiderBond::get()
                + <Runtime as Config>::OracleBond::get()
        );
        // DAVE gets his outcome bond back
        assert_eq!(Balances::free_balance(DAVE), dave_balance_before + outcome_bond);
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
