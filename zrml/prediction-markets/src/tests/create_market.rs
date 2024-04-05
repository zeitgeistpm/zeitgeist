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

use crate::MarketBondsOf;
use core::ops::RangeInclusive;
use zeitgeist_primitives::{
    constants::MILLISECS_PER_BLOCK,
    types::{BlockNumber, Bond, MarketBonds, Moment},
};

#[test_case(
    MarketCreation::Advised,
    <Runtime as Config>::AdvisoryBond::get() + <Runtime as Config>::OracleBond::get() - 1
)]
#[test_case(
    MarketCreation::Permissionless,
    <Runtime as Config>::ValidityBond::get() + <Runtime as Config>::OracleBond::get() - 1
)]
fn fails_if_user_cannot_afford_bonds_advised(
    market_creation: MarketCreation,
    balance: BalanceOf<Runtime>,
) {
    ExtBuilder::default().build().execute_with(|| {
        let creator = 999;
        assert_ok!(AssetManager::deposit(Asset::Ztg, &creator, balance));
        assert_noop!(
            PredictionMarkets::create_market(
                RuntimeOrigin::signed(creator),
                BaseAsset::Ztg,
                <Runtime as Config>::MaxCreatorFee::get(),
                BOB,
                MarketPeriod::Block(123..456),
                get_deadlines(),
                gen_metadata(2),
                market_creation,
                MarketType::Scalar(0..=1),
                Some(MarketDisputeMechanism::SimpleDisputes),
                ScoringRule::Lmsr,
            ),
            pallet_balances::Error::<Runtime>::InsufficientBalance
        );
    });
}

#[test]
fn fails_on_fee_too_high() {
    ExtBuilder::default().build().execute_with(|| {
        let one_billionth = Perbill::from_rational(1u128, 1_000_000_000u128);
        assert_noop!(
            PredictionMarkets::create_market(
                RuntimeOrigin::signed(ALICE),
                BaseAsset::Ztg,
                <Runtime as Config>::MaxCreatorFee::get() + one_billionth,
                BOB,
                MarketPeriod::Block(123..456),
                get_deadlines(),
                gen_metadata(2),
                MarketCreation::Permissionless,
                MarketType::Scalar(0..=1),
                Some(MarketDisputeMechanism::SimpleDisputes),
                ScoringRule::Lmsr,
            ),
            Error::<Runtime>::FeeTooHigh
        );
    });
}

#[test]
fn fails_on_invalid_multihash() {
    ExtBuilder::default().build().execute_with(|| {
        let mut metadata = [0xff; 50];
        metadata[0] = 0x15;
        metadata[1] = 0x29;
        let multihash = MultiHash::Sha3_384(metadata);
        assert_noop!(
            PredictionMarkets::create_market(
                RuntimeOrigin::signed(ALICE),
                BaseAsset::Ztg,
                <Runtime as Config>::MaxCreatorFee::get(),
                BOB,
                MarketPeriod::Block(123..456),
                get_deadlines(),
                multihash,
                MarketCreation::Permissionless,
                MarketType::Scalar(0..=1),
                Some(MarketDisputeMechanism::SimpleDisputes),
                ScoringRule::Lmsr,
            ),
            Error::<Runtime>::InvalidMultihash
        );
    });
}

#[test_case(std::ops::RangeInclusive::new(7, 6); "empty range")]
#[test_case(555..=555; "one element as range")]
fn create_scalar_market_fails_on_invalid_range(range: RangeInclusive<u128>) {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PredictionMarkets::create_market(
                RuntimeOrigin::signed(ALICE),
                BaseAsset::Ztg,
                Perbill::zero(),
                BOB,
                MarketPeriod::Block(123..456),
                get_deadlines(),
                gen_metadata(2),
                MarketCreation::Permissionless,
                MarketType::Scalar(range),
                Some(MarketDisputeMechanism::SimpleDisputes),
                ScoringRule::Lmsr,
            ),
            Error::<Runtime>::InvalidOutcomeRange
        );
    });
}

#[test]
fn create_market_fails_on_min_dispute_period() {
    ExtBuilder::default().build().execute_with(|| {
        let deadlines = Deadlines {
            grace_period: <Runtime as Config>::MaxGracePeriod::get(),
            oracle_duration: <Runtime as Config>::MaxOracleDuration::get(),
            dispute_duration: <Runtime as Config>::MinDisputeDuration::get() - 1,
        };
        assert_noop!(
            PredictionMarkets::create_market(
                RuntimeOrigin::signed(ALICE),
                BaseAsset::Ztg,
                Perbill::zero(),
                BOB,
                MarketPeriod::Block(123..456),
                deadlines,
                gen_metadata(2),
                MarketCreation::Permissionless,
                MarketType::Categorical(2),
                Some(MarketDisputeMechanism::SimpleDisputes),
                ScoringRule::Lmsr,
            ),
            Error::<Runtime>::DisputeDurationSmallerThanMinDisputeDuration
        );
    });
}

#[test]
fn create_market_fails_on_min_oracle_duration() {
    ExtBuilder::default().build().execute_with(|| {
        let deadlines = Deadlines {
            grace_period: <Runtime as Config>::MaxGracePeriod::get(),
            oracle_duration: <Runtime as Config>::MinOracleDuration::get() - 1,
            dispute_duration: <Runtime as Config>::MinDisputeDuration::get(),
        };
        assert_noop!(
            PredictionMarkets::create_market(
                RuntimeOrigin::signed(ALICE),
                BaseAsset::Ztg,
                Perbill::zero(),
                BOB,
                MarketPeriod::Block(123..456),
                deadlines,
                gen_metadata(2),
                MarketCreation::Permissionless,
                MarketType::Categorical(2),
                Some(MarketDisputeMechanism::SimpleDisputes),
                ScoringRule::Lmsr,
            ),
            Error::<Runtime>::OracleDurationSmallerThanMinOracleDuration
        );
    });
}

#[test]
fn create_market_fails_on_max_dispute_period() {
    ExtBuilder::default().build().execute_with(|| {
        let deadlines = Deadlines {
            grace_period: <Runtime as Config>::MaxGracePeriod::get(),
            oracle_duration: <Runtime as Config>::MaxOracleDuration::get(),
            dispute_duration: <Runtime as Config>::MaxDisputeDuration::get() + 1,
        };
        assert_noop!(
            PredictionMarkets::create_market(
                RuntimeOrigin::signed(ALICE),
                BaseAsset::Ztg,
                Perbill::zero(),
                BOB,
                MarketPeriod::Block(123..456),
                deadlines,
                gen_metadata(2),
                MarketCreation::Permissionless,
                MarketType::Categorical(2),
                Some(MarketDisputeMechanism::SimpleDisputes),
                ScoringRule::Lmsr,
            ),
            Error::<Runtime>::DisputeDurationGreaterThanMaxDisputeDuration
        );
    });
}

#[test]
fn create_market_fails_on_max_grace_period() {
    ExtBuilder::default().build().execute_with(|| {
        let deadlines = Deadlines {
            grace_period: <Runtime as Config>::MaxGracePeriod::get() + 1,
            oracle_duration: <Runtime as Config>::MaxOracleDuration::get(),
            dispute_duration: <Runtime as Config>::MaxDisputeDuration::get(),
        };
        assert_noop!(
            PredictionMarkets::create_market(
                RuntimeOrigin::signed(ALICE),
                BaseAsset::Ztg,
                Perbill::zero(),
                BOB,
                MarketPeriod::Block(123..456),
                deadlines,
                gen_metadata(2),
                MarketCreation::Permissionless,
                MarketType::Categorical(2),
                Some(MarketDisputeMechanism::SimpleDisputes),
                ScoringRule::Lmsr,
            ),
            Error::<Runtime>::GracePeriodGreaterThanMaxGracePeriod
        );
    });
}

#[test]
fn create_market_fails_on_max_oracle_duration() {
    ExtBuilder::default().build().execute_with(|| {
        let deadlines = Deadlines {
            grace_period: <Runtime as Config>::MaxGracePeriod::get(),
            oracle_duration: <Runtime as Config>::MaxOracleDuration::get() + 1,
            dispute_duration: <Runtime as Config>::MaxDisputeDuration::get(),
        };
        assert_noop!(
            PredictionMarkets::create_market(
                RuntimeOrigin::signed(ALICE),
                BaseAsset::Ztg,
                Perbill::zero(),
                BOB,
                MarketPeriod::Block(123..456),
                deadlines,
                gen_metadata(2),
                MarketCreation::Permissionless,
                MarketType::Categorical(2),
                Some(MarketDisputeMechanism::SimpleDisputes),
                ScoringRule::Lmsr,
            ),
            Error::<Runtime>::OracleDurationGreaterThanMaxOracleDuration
        );
    });
}

// TODO(#1239) split this test
#[cfg(feature = "parachain")]
#[test]
fn create_market_with_foreign_assets() {
    ExtBuilder::default().build().execute_with(|| {
        let deadlines = Deadlines {
            grace_period: <Runtime as Config>::MaxGracePeriod::get(),
            oracle_duration: <Runtime as Config>::MaxOracleDuration::get(),
            dispute_duration: <Runtime as Config>::MaxDisputeDuration::get(),
        };

        // As per Mock asset_registry genesis ForeignAsset(420) has allow_as_base_asset set to false.

        assert_noop!(
            PredictionMarkets::create_market(
                RuntimeOrigin::signed(ALICE),
                BaseAsset::ForeignAsset(420),
                Perbill::zero(),
                BOB,
                MarketPeriod::Block(123..456),
                deadlines,
                gen_metadata(2),
                MarketCreation::Permissionless,
                MarketType::Categorical(2),
                Some(MarketDisputeMechanism::SimpleDisputes),
                ScoringRule::Lmsr,
            ),
            Error::<Runtime>::InvalidBaseAsset,
        );
        // As per Mock asset_registry genesis ForeignAsset(50) is not registered in asset_registry.
        assert_noop!(
            PredictionMarkets::create_market(
                RuntimeOrigin::signed(ALICE),
                BaseAsset::ForeignAsset(50),
                Perbill::zero(),
                BOB,
                MarketPeriod::Block(123..456),
                deadlines,
                gen_metadata(2),
                MarketCreation::Permissionless,
                MarketType::Categorical(2),
                Some(MarketDisputeMechanism::SimpleDisputes),
                ScoringRule::Lmsr,
            ),
            Error::<Runtime>::UnregisteredForeignAsset,
        );
        // As per Mock asset_registry genesis ForeignAsset(100) has allow_as_base_asset set to true.
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::ForeignAsset(100),
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(123..456),
            deadlines,
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.base_asset, BaseAsset::ForeignAsset(100));
    });
}

#[test]
fn it_does_not_create_market_with_too_few_categories() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PredictionMarkets::create_market(
                RuntimeOrigin::signed(ALICE),
                BaseAsset::Ztg,
                Perbill::zero(),
                BOB,
                MarketPeriod::Block(0..100),
                get_deadlines(),
                gen_metadata(2),
                MarketCreation::Advised,
                MarketType::Categorical(<Runtime as Config>::MinCategories::get() - 1),
                Some(MarketDisputeMechanism::SimpleDisputes),
                ScoringRule::Lmsr
            ),
            Error::<Runtime>::NotEnoughCategories
        );
    });
}

#[test]
fn it_does_not_create_market_with_too_many_categories() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PredictionMarkets::create_market(
                RuntimeOrigin::signed(ALICE),
                BaseAsset::Ztg,
                Perbill::zero(),
                BOB,
                MarketPeriod::Block(0..100),
                get_deadlines(),
                gen_metadata(2),
                MarketCreation::Advised,
                MarketType::Categorical(<Runtime as Config>::MaxCategories::get() + 1),
                Some(MarketDisputeMechanism::SimpleDisputes),
                ScoringRule::Lmsr
            ),
            Error::<Runtime>::TooManyCategories
        );
    });
}

#[test_case(MarketPeriod::Block(3..3); "empty range blocks")]
#[test_case(MarketPeriod::Timestamp(3..3); "empty range timestamp")]
#[test_case(
    MarketPeriod::Timestamp(0..(MILLISECS_PER_BLOCK - 1).into());
    "range shorter than block time"
)]
fn create_categorical_market_fails_if_market_period_is_invalid(
    period: MarketPeriod<BlockNumber, Moment>,
) {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PredictionMarkets::create_market(
                RuntimeOrigin::signed(ALICE),
                BaseAsset::Ztg,
                Perbill::zero(),
                BOB,
                period,
                get_deadlines(),
                gen_metadata(0),
                MarketCreation::Permissionless,
                MarketType::Categorical(3),
                Some(MarketDisputeMechanism::Authorized),
                ScoringRule::Lmsr,
            ),
            Error::<Runtime>::InvalidMarketPeriod,
        );
    });
}

#[test]
fn create_categorical_market_fails_if_end_is_not_far_enough_ahead() {
    ExtBuilder::default().build().execute_with(|| {
        let end_block = 33;
        run_to_block(end_block);
        assert_noop!(
            PredictionMarkets::create_market(
                RuntimeOrigin::signed(ALICE),
                BaseAsset::Ztg,
                Perbill::zero(),
                BOB,
                MarketPeriod::Block(0..end_block),
                get_deadlines(),
                gen_metadata(0),
                MarketCreation::Permissionless,
                MarketType::Categorical(3),
                Some(MarketDisputeMechanism::Authorized),
                ScoringRule::Lmsr,
            ),
            Error::<Runtime>::InvalidMarketPeriod,
        );

        let end_time = MILLISECS_PER_BLOCK as u64 / 2;
        assert_noop!(
            PredictionMarkets::create_market(
                RuntimeOrigin::signed(ALICE),
                BaseAsset::Ztg,
                Perbill::zero(),
                BOB,
                MarketPeriod::Timestamp(0..end_time),
                get_deadlines(),
                gen_metadata(0),
                MarketCreation::Permissionless,
                MarketType::Categorical(3),
                Some(MarketDisputeMechanism::Authorized),
                ScoringRule::Lmsr,
            ),
            Error::<Runtime>::InvalidMarketPeriod,
        );
    });
}

#[test]
fn create_market_succeeds_if_market_duration_is_maximal_in_blocks() {
    ExtBuilder::default().build().execute_with(|| {
        let now = 1;
        frame_system::Pallet::<Runtime>::set_block_number(now);
        let start = 5;
        let end = now + <Runtime as Config>::MaxMarketLifetime::get();
        assert!(
            end > start,
            "Test failed due to misconfiguration: `MaxMarketLifetime` is too small"
        );
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(start..end),
            get_deadlines(),
            gen_metadata(0),
            MarketCreation::Permissionless,
            MarketType::Categorical(3),
            Some(MarketDisputeMechanism::Authorized),
            ScoringRule::Lmsr,
        ));
    });
}

#[test]
fn create_market_suceeds_if_market_duration_is_maximal_in_moments() {
    ExtBuilder::default().build().execute_with(|| {
        let now = 12_001u64;
        Timestamp::set_timestamp(now);
        let start = 5 * MILLISECS_PER_BLOCK as u64;
        let end =
            now + <Runtime as Config>::MaxMarketLifetime::get() * (MILLISECS_PER_BLOCK as u64);
        assert!(
            end > start,
            "Test failed due to misconfiguration: `MaxMarketLifetime` is too small"
        );
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Timestamp(start..end),
            get_deadlines(),
            gen_metadata(0),
            MarketCreation::Permissionless,
            MarketType::Categorical(3),
            Some(MarketDisputeMechanism::Authorized),
            ScoringRule::Lmsr,
        ));
    });
}

#[test]
fn create_market_fails_if_market_duration_is_too_long_in_blocks() {
    ExtBuilder::default().build().execute_with(|| {
        let now = 1;
        frame_system::Pallet::<Runtime>::set_block_number(now);
        let start = 5;
        let end = now + <Runtime as Config>::MaxMarketLifetime::get() + 1;
        assert!(
            end > start,
            "Test failed due to misconfiguration: `MaxMarketLifetime` is too small"
        );
        assert_noop!(
            PredictionMarkets::create_market(
                RuntimeOrigin::signed(ALICE),
                BaseAsset::Ztg,
                Perbill::zero(),
                BOB,
                MarketPeriod::Block(start..end),
                get_deadlines(),
                gen_metadata(0),
                MarketCreation::Permissionless,
                MarketType::Categorical(3),
                Some(MarketDisputeMechanism::Authorized),
                ScoringRule::Lmsr,
            ),
            crate::Error::<Runtime>::MarketDurationTooLong,
        );
    });
}

#[test]
fn create_market_fails_if_market_duration_is_too_long_in_moments() {
    ExtBuilder::default().build().execute_with(|| {
        let now = 12_001u64;
        Timestamp::set_timestamp(now);
        let start = 5 * MILLISECS_PER_BLOCK as u64;
        let end = now
            + (<Runtime as Config>::MaxMarketLifetime::get() + 1) * (MILLISECS_PER_BLOCK as u64);
        assert!(
            end > start,
            "Test failed due to misconfiguration: `MaxMarketLifetime` is too small"
        );
        assert_noop!(
            PredictionMarkets::create_market(
                RuntimeOrigin::signed(ALICE),
                BaseAsset::Ztg,
                Perbill::zero(),
                BOB,
                MarketPeriod::Timestamp(start..end),
                get_deadlines(),
                gen_metadata(0),
                MarketCreation::Permissionless,
                MarketType::Categorical(3),
                Some(MarketDisputeMechanism::Authorized),
                ScoringRule::Lmsr,
            ),
            crate::Error::<Runtime>::MarketDurationTooLong,
        );
    });
}

#[test_case(
    MarketCreation::Advised,
    ScoringRule::Lmsr,
    MarketStatus::Proposed,
    MarketBonds {
        creation: Some(Bond::new(ALICE, <Runtime as Config>::AdvisoryBond::get())),
        oracle: Some(Bond::new(ALICE, <Runtime as Config>::OracleBond::get())),
        outsider: None,
        dispute: None,
        close_dispute: None,
        close_request: None,
    }
)]
#[test_case(
    MarketCreation::Permissionless,
    ScoringRule::Lmsr,
    MarketStatus::Active,
    MarketBonds {
        creation: Some(Bond::new(ALICE, <Runtime as Config>::ValidityBond::get())),
        oracle: Some(Bond::new(ALICE, <Runtime as Config>::OracleBond::get())),
        outsider: None,
        dispute: None,
        close_dispute: None,
        close_request: None,
    }
)]
fn create_market_sets_the_correct_market_parameters_and_reserves_the_correct_amount(
    creation: MarketCreation,
    scoring_rule: ScoringRule,
    status: MarketStatus,
    bonds: MarketBondsOf<Runtime>,
) {
    ExtBuilder::default().build().execute_with(|| {
        let creator = ALICE;
        let oracle = BOB;
        let period = MarketPeriod::Block(1..2);
        let deadlines = Deadlines {
            grace_period: 1,
            oracle_duration: <Runtime as Config>::MinOracleDuration::get() + 2,
            dispute_duration: <Runtime as Config>::MinDisputeDuration::get() + 3,
        };
        let metadata = gen_metadata(0x99);
        let MultiHash::Sha3_384(multihash) = metadata;
        let market_type = MarketType::Categorical(7);
        let dispute_mechanism = Some(MarketDisputeMechanism::Authorized);
        let creator_fee = Perbill::from_parts(1);
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(creator),
            BaseAsset::Ztg,
            creator_fee,
            oracle,
            period.clone(),
            deadlines,
            metadata,
            creation.clone(),
            market_type.clone(),
            dispute_mechanism.clone(),
            scoring_rule,
        ));
        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.creator, creator);
        assert_eq!(market.creation, creation);
        assert_eq!(market.creator_fee, creator_fee);
        assert_eq!(market.oracle, oracle);
        assert_eq!(market.metadata, multihash);
        assert_eq!(market.market_type, market_type);
        assert_eq!(market.period, period);
        assert_eq!(market.deadlines, deadlines);
        assert_eq!(market.scoring_rule, scoring_rule);
        assert_eq!(market.status, status);
        assert_eq!(market.report, None);
        assert_eq!(market.resolved_outcome, None);
        assert_eq!(market.dispute_mechanism, dispute_mechanism);
        assert_eq!(market.bonds, bonds);
    });
}

#[test]
fn create_market_fails_on_trusted_market_with_non_zero_dispute_period() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PredictionMarkets::create_market(
                RuntimeOrigin::signed(ALICE),
                BaseAsset::Ztg,
                Perbill::zero(),
                BOB,
                MarketPeriod::Block(1..2),
                Deadlines {
                    grace_period: 1,
                    oracle_duration: <Runtime as Config>::MinOracleDuration::get() + 2,
                    dispute_duration: <Runtime as Config>::MinDisputeDuration::get() + 3,
                },
                gen_metadata(0x99),
                MarketCreation::Permissionless,
                MarketType::Categorical(3),
                None,
                ScoringRule::Lmsr,
            ),
            Error::<Runtime>::NonZeroDisputePeriodOnTrustedMarket
        );
    });
}

#[test]
fn create_categorical_market_deposits_the_correct_event() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            BaseAsset::Ztg,
            MarketCreation::Permissionless,
            1..2,
            ScoringRule::Lmsr,
        );
        let market_id = 0;
        let market = MarketCommons::market(&market_id).unwrap();
        let market_account = PredictionMarkets::market_account(market_id);
        System::assert_last_event(Event::MarketCreated(0, market_account, market).into());
    });
}

#[test]
fn create_scalar_market_deposits_the_correct_event() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_scalar_market(
            BaseAsset::Ztg,
            MarketCreation::Permissionless,
            1..2,
            ScoringRule::Lmsr,
        );
        let market_id = 0;
        let market = MarketCommons::market(&market_id).unwrap();
        let market_account = PredictionMarkets::market_account(market_id);
        System::assert_last_event(Event::MarketCreated(0, market_account, market).into());
    });
}

#[test]
fn does_trigger_market_transition_api() {
    ExtBuilder::default().build().execute_with(|| {
        StateTransitionMock::ensure_empty_state();
        simple_create_categorical_market(
            BaseAsset::Ztg,
            MarketCreation::Advised,
            1..2,
            ScoringRule::Lmsr,
        );
        assert!(StateTransitionMock::on_proposal_triggered());
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
            ScoringRule::Lmsr,
        );
        assert!(StateTransitionMock::on_activation_triggered());
    });
}
