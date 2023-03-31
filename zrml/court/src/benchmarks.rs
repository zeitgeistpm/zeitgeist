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
    clippy::integer_arithmetic
)]
#![cfg(feature = "runtime-benchmarks")]

extern crate alloc;
use crate::{
    types::{CourtStatus, Draw, JurorInfo, JurorPoolItem, Vote},
    AppealInfo, BalanceOf, Call, Config, Courts, Draws, JurorPool, Jurors, MarketOf,
    Pallet as Court, Pallet, RequestBlock,
};
use alloc::vec;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use sp_runtime::{
    traits::{Bounded, Hash, Saturating, StaticLookup, Zero},
    SaturatedConversion,
};
use zeitgeist_primitives::{
    traits::{DisputeApi, DisputeResolutionApi},
    types::{
        Asset, Deadlines, Market, MarketBonds, MarketCreation, MarketDisputeMechanism,
        MarketPeriod, MarketStatus, MarketType, OutcomeReport, Report, ScoringRule,
    },
};
use zrml_market_commons::MarketCommonsPalletApi;

const ORACLE_REPORT: OutcomeReport = OutcomeReport::Scalar(u128::MAX);

fn get_market<T>() -> MarketOf<T>
where
    T: Config,
{
    Market {
        base_asset: Asset::Ztg,
        creation: MarketCreation::Permissionless,
        creator_fee: 0,
        creator: account("creator", 0, 0),
        market_type: MarketType::Scalar(0..=100),
        dispute_mechanism: MarketDisputeMechanism::Court,
        metadata: vec![],
        oracle: account("oracle", 0, 0),
        period: MarketPeriod::Block(
            0u64.saturated_into::<T::BlockNumber>()..100u64.saturated_into::<T::BlockNumber>(),
        ),
        deadlines: Deadlines {
            grace_period: 1_u64.saturated_into::<T::BlockNumber>(),
            oracle_duration: 1_u64.saturated_into::<T::BlockNumber>(),
            dispute_duration: 1_u64.saturated_into::<T::BlockNumber>(),
        },
        report: Some(Report {
            at: 1u64.saturated_into::<T::BlockNumber>(),
            by: account("oracle", 0, 0),
            outcome: ORACLE_REPORT,
        }),
        resolved_outcome: None,
        status: MarketStatus::Disputed,
        scoring_rule: ScoringRule::CPMM,
        bonds: MarketBonds { creation: None, oracle: None, outsider: None, dispute: None },
    }
}

fn deposit<T>(caller: &T::AccountId)
where
    T: Config,
{
    let _ = T::Currency::deposit_creating(caller, BalanceOf::<T>::max_value());
}

fn fill_pool<T>(number: u32) -> Result<(), &'static str>
where
    T: Config,
{
    let mut jurors = <JurorPool<T>>::get();
    let min_amount = T::MinJurorStake::get();
    let max_amount = min_amount + min_amount + BalanceOf::<T>::from(number);
    for i in 0..number {
        let juror: T::AccountId = account("juror", i, 0);
        let stake = max_amount - BalanceOf::<T>::from(i);
        <Jurors<T>>::insert(
            juror.clone(),
            JurorInfo { stake, active_lock: <BalanceOf<T>>::zero() },
        );
        let consumed_stake = BalanceOf::<T>::zero();
        let pool_item = JurorPoolItem { stake, juror: juror.clone(), consumed_stake };
        match jurors
            .binary_search_by_key(&(stake, &juror), |pool_item| (pool_item.stake, &pool_item.juror))
        {
            Ok(_) => panic!("Juror already in pool"),
            Err(index) => jurors.try_insert(index, pool_item).unwrap(),
        };
    }
    <JurorPool<T>>::put(jurors);
    Ok(())
}

fn join_with_min_stake<T>(caller: &T::AccountId) -> Result<(), &'static str>
where
    T: Config,
{
    let stake = T::MinJurorStake::get();
    deposit::<T>(caller);
    Court::<T>::join_court(RawOrigin::Signed(caller.clone()).into(), stake)?;
    Ok(())
}

fn setup_court<T>() -> Result<crate::MarketIdOf<T>, &'static str>
where
    T: Config,
{
    <frame_system::Pallet<T>>::set_block_number(1u64.saturated_into::<T::BlockNumber>());

    let now = <frame_system::Pallet<T>>::block_number();
    <RequestBlock<T>>::put(now + 1u64.saturated_into::<T::BlockNumber>());

    let market_id = T::MarketCommons::push_market(get_market::<T>()).unwrap();
    Court::<T>::on_dispute(&market_id, &get_market::<T>()).unwrap();

    Ok(market_id)
}

fn fill_draws<T>(market_id: crate::MarketIdOf<T>, number: u32) -> Result<(), &'static str>
where
    T: Config,
{
    // remove last random selections of on_dispute
    <Draws<T>>::remove(market_id);
    let mut draws = <Draws<T>>::get(market_id);
    for i in 0..number {
        let juror = account("juror", i, 0);
        deposit::<T>(&juror);
        <Jurors<T>>::insert(
            &juror,
            JurorInfo { stake: T::MinJurorStake::get(), active_lock: T::MinJurorStake::get() },
        );
        let draw =
            Draw { juror, vote: Vote::Drawn, weight: 1u32, slashable: T::MinJurorStake::get() };
        draws.try_push(draw).unwrap();
    }
    <Draws<T>>::insert(market_id, draws);
    Ok(())
}

benchmarks! {
    join_court {
        let j in 0..(T::MaxJurors::get() - 1);

        fill_pool::<T>(j)?;

        let caller: T::AccountId = whitelisted_caller();
        join_with_min_stake::<T>(&caller)?;

        let new_stake = T::MinJurorStake::get()
            .saturating_add(1u128.saturated_into::<BalanceOf<T>>());
    }: _(RawOrigin::Signed(caller), new_stake)

    prepare_exit_court {
        let j in 0..(T::MaxJurors::get() - 1);

        fill_pool::<T>(j)?;

        let caller: T::AccountId = whitelisted_caller();
        join_with_min_stake::<T>(&caller)?;
    }: _(RawOrigin::Signed(caller))

    exit_court_remove {
        let j in 0..(T::MaxJurors::get() - 1);

        fill_pool::<T>(j)?;

        let caller: T::AccountId = whitelisted_caller();
        join_with_min_stake::<T>(&caller)?;

        Court::<T>::prepare_exit_court(RawOrigin::Signed(caller.clone()).into())?;

        <Jurors<T>>::mutate(caller.clone(), |prev_juror_info| {
            prev_juror_info.as_mut().unwrap().active_lock = <BalanceOf<T>>::zero();
        });

        let juror = T::Lookup::unlookup(caller.clone());
    }: exit_court(RawOrigin::Signed(caller), juror)

    exit_court_set {
        let j in 0..(T::MaxJurors::get() - 1);

        fill_pool::<T>(j)?;

        let caller: T::AccountId = whitelisted_caller();
        join_with_min_stake::<T>(&caller)?;

        Court::<T>::prepare_exit_court(RawOrigin::Signed(caller.clone()).into())?;

        <Jurors<T>>::mutate(caller.clone(), |prev_juror_info| {
            prev_juror_info.as_mut().unwrap().active_lock = T::MinJurorStake::get();
        });

        let juror = T::Lookup::unlookup(caller.clone());
    }: exit_court(RawOrigin::Signed(caller), juror)

    vote {
        let d in 1..T::MaxDraws::get();

        fill_pool::<T>(T::MaxJurors::get() - 1)?;

        let caller: T::AccountId = whitelisted_caller();
        let market_id = setup_court::<T>()?;

        let court = <Courts<T>>::get(market_id).unwrap();
        let pre_vote_end = court.periods.pre_vote_end;

        fill_draws::<T>(market_id, d)?;

        let mut draws = <Draws<T>>::get(market_id);
        let draws_len = draws.len();
        draws[draws_len.saturating_sub(1usize)] = Draw {
            juror: caller.clone(),
            vote: Vote::Drawn,
            weight: 1u32,
            slashable: <BalanceOf<T>>::zero()
        };
        <Draws<T>>::insert(market_id, draws);

        <frame_system::Pallet<T>>::set_block_number(pre_vote_end + 1u64.saturated_into::<T::BlockNumber>());

        let commitment_vote = Default::default();
    }: _(RawOrigin::Signed(caller), market_id, commitment_vote)

    denounce_vote {
        let d in 1..T::MaxDraws::get();

        let necessary_jurors_weight: usize = Court::<T>::necessary_jurors_weight(0usize);
        fill_pool::<T>(necessary_jurors_weight as u32)?;

        let caller: T::AccountId = whitelisted_caller();
        let market_id = setup_court::<T>()?;

        let court = <Courts<T>>::get(market_id).unwrap();
        let pre_vote_end = court.periods.pre_vote_end;

        fill_draws::<T>(market_id, d)?;

        let salt = Default::default();
        let outcome = OutcomeReport::Scalar(0u128);
        let denounced_juror: T::AccountId = account("denounced_juror", 0, 0);
        join_with_min_stake::<T>(&denounced_juror)?;
        <Jurors<T>>::insert(&denounced_juror, JurorInfo {
            stake: T::MinJurorStake::get(),
            active_lock: T::MinJurorStake::get(),
        });
        let denounced_juror_unlookup = T::Lookup::unlookup(denounced_juror.clone());
        let commitment = T::Hashing::hash_of(&(denounced_juror.clone(), outcome.clone(), salt));

        let mut draws = <Draws<T>>::get(market_id);
        let draws_len = draws.len();
        draws[draws_len.saturating_sub(1usize)] = Draw {
            juror: denounced_juror,
            vote: Vote::Secret { commitment },
            weight: 1u32,
            slashable: T::MinJurorStake::get(),
        };
        <Draws<T>>::insert(market_id, draws);

        <frame_system::Pallet<T>>::set_block_number(pre_vote_end + 1u64.saturated_into::<T::BlockNumber>());
    }: _(RawOrigin::Signed(caller), market_id, denounced_juror_unlookup, outcome, salt)

    reveal_vote {
        let d in 1..T::MaxDraws::get();

        fill_pool::<T>(T::MaxJurors::get() - 1)?;

        let caller: T::AccountId = whitelisted_caller();
        let market_id = setup_court::<T>()?;

        let court = <Courts<T>>::get(market_id).unwrap();
        let vote_end = court.periods.vote_end;

        fill_draws::<T>(market_id, d)?;

        let salt = Default::default();
        let outcome = OutcomeReport::Scalar(0u128);
        join_with_min_stake::<T>(&caller)?;
        <Jurors<T>>::insert(&caller, JurorInfo {
            stake: T::MinJurorStake::get(),
            active_lock: T::MinJurorStake::get(),
        });
        let commitment = T::Hashing::hash_of(&(caller.clone(), outcome.clone(), salt));

        let mut draws = <Draws<T>>::get(market_id);
        let draws_len = draws.len();
        draws[draws_len.saturating_sub(1usize)] = Draw {
            juror: caller.clone(),
            vote: Vote::Secret { commitment },
            weight: 1u32,
            slashable: T::MinJurorStake::get(),
        };
        <Draws<T>>::insert(market_id, draws);

        <frame_system::Pallet<T>>::set_block_number(vote_end + 1u64.saturated_into::<T::BlockNumber>());
    }: _(RawOrigin::Signed(caller), market_id, outcome, salt)

    appeal {
        // from 47 because in the last appeal round we need at least 47 jurors
        let j in 47..T::MaxJurors::get();
        let a in 0..(T::MaxAppeals::get() - 2);
        let r in 0..62;
        let f in 0..62;

        let necessary_jurors_weight = Court::<T>::necessary_jurors_weight((T::MaxAppeals::get() - 1) as usize);
        debug_assert!(necessary_jurors_weight == 47usize);
        fill_pool::<T>(j)?;

        let caller: T::AccountId = whitelisted_caller();
        deposit::<T>(&caller);
        let market_id = setup_court::<T>()?;

        let mut court = <Courts<T>>::get(market_id).unwrap();
        let appeal_end = court.periods.appeal_end;
        for i in 0..r {
            let market_id_i = (i + 100).saturated_into::<crate::MarketIdOf<T>>();
            T::DisputeResolution::add_auto_resolve(&market_id_i, appeal_end).unwrap();
        }
        T::DisputeResolution::add_auto_resolve(&market_id, appeal_end).unwrap();

        let aggregation_end = court.periods.aggregation_end;
        for i in 0..a {
            let appeal_info = AppealInfo {
                backer: account("backer", i, 0),
                bond: crate::default_appeal_bond::<T>(i as usize),
                appealed_outcome: OutcomeReport::Scalar(0u128),
            };
            court.appeals.try_push(appeal_info).unwrap();
        }
        <Courts<T>>::insert(market_id, court);

        let salt = Default::default();
        // remove last random selections of on_dispute
        <Draws<T>>::remove(market_id);
        let mut draws = <Draws<T>>::get(market_id);
        let draws_len = Court::<T>::necessary_jurors_weight(a as usize) as u32;
        for i in 0..draws_len {
            let juror: T::AccountId = account("juror", i, 0);
            <Jurors<T>>::insert(&juror, JurorInfo {
                stake: T::MinJurorStake::get(),
                active_lock: T::MinJurorStake::get(),
            });
            let outcome = OutcomeReport::Scalar(i as u128);
            let commitment = T::Hashing::hash_of(&(juror.clone(), outcome.clone(), salt));
            let draw =
                Draw {
                    juror,
                    vote: Vote::Revealed { commitment, outcome, salt },
                    weight: 1u32,
                    slashable: <BalanceOf<T>>::zero()
                };
            draws.try_push(draw).unwrap();
        }
        <Draws<T>>::insert(market_id, draws);

        <frame_system::Pallet<T>>::set_block_number(aggregation_end + 1u64.saturated_into::<T::BlockNumber>());
        let now = <frame_system::Pallet<T>>::block_number();
        <RequestBlock<T>>::put(now + 1u64.saturated_into::<T::BlockNumber>());

        let new_resolve_at = <RequestBlock<T>>::get()
            + T::CourtVotePeriod::get()
            + T::CourtAggregationPeriod::get()
            + T::CourtAppealPeriod::get();
        for i in 0..f {
            let market_id_i = (i + 100).saturated_into::<crate::MarketIdOf<T>>();
            T::DisputeResolution::add_auto_resolve(&market_id_i, new_resolve_at).unwrap();
        }
    }: _(RawOrigin::Signed(caller), market_id)
    verify {
        let court = <Courts<T>>::get(market_id).unwrap();
        assert_eq!(court.periods.appeal_end, new_resolve_at);
    }

    back_global_dispute {
        let d in 1..T::MaxDraws::get();
        let r in 0..62;

        let necessary_jurors_weight: usize = Court::<T>::necessary_jurors_weight(0usize);
        fill_pool::<T>(necessary_jurors_weight as u32)?;

        let caller: T::AccountId = whitelisted_caller();
        deposit::<T>(&caller);
        let market_id = setup_court::<T>()?;

        let mut court = <Courts<T>>::get(market_id).unwrap();
        let appeal_end = court.periods.appeal_end;
        for i in 0..r {
            let market_id_i = i.saturated_into::<crate::MarketIdOf<T>>();
            T::DisputeResolution::add_auto_resolve(&market_id_i, appeal_end).unwrap();
        }

        T::DisputeResolution::add_auto_resolve(&market_id, appeal_end).unwrap();

        let aggregation_end = court.periods.aggregation_end;
        for i in 0..(T::MaxAppeals::get() - 1) {
            let appeal_info = AppealInfo {
                backer: account("backer", i, 0),
                bond: crate::default_appeal_bond::<T>(i as usize),
                appealed_outcome: OutcomeReport::Scalar(0u128),
            };
            court.appeals.try_push(appeal_info).unwrap();
        }
        <Courts<T>>::insert(market_id, court);

        let salt = Default::default();
        // remove last random selections of on_dispute
        <Draws<T>>::remove(market_id);
        let mut draws = <Draws<T>>::get(market_id);
        for i in 0..d {
            let juror: T::AccountId = account("juror", i, 0);
            <Jurors<T>>::insert(&juror, JurorInfo {
                stake: T::MinJurorStake::get(),
                active_lock: T::MinJurorStake::get(),
            });
            let outcome = OutcomeReport::Scalar(i as u128);
            let commitment = T::Hashing::hash_of(&(juror.clone(), outcome.clone(), salt));
            let draw =
                Draw {
                    juror,
                    vote: Vote::Revealed { commitment, outcome, salt },
                    weight: 1u32,
                    slashable: T::MinJurorStake::get(),
                };
            draws.try_push(draw).unwrap();
        }
        <Draws<T>>::insert(market_id, draws);

        <frame_system::Pallet<T>>::set_block_number(aggregation_end + 1u64.saturated_into::<T::BlockNumber>());
    }: _(RawOrigin::Signed(caller), market_id)

    reassign_juror_stakes {
        let d in 1..T::MaxDraws::get();

        let necessary_jurors_weight: usize = Court::<T>::necessary_jurors_weight(0usize);
        fill_pool::<T>(necessary_jurors_weight as u32)?;

        let caller: T::AccountId = whitelisted_caller();
        let market_id = setup_court::<T>()?;

        let mut court = <Courts<T>>::get(market_id).unwrap();
        let winner_outcome = OutcomeReport::Scalar(0u128);
        court.status = CourtStatus::Closed { winner: winner_outcome.clone() };
        <Courts<T>>::insert(market_id, court);

        let salt = Default::default();
        // remove last random selections of on_dispute
        <Draws<T>>::remove(market_id);
        let mut draws = <Draws<T>>::get(market_id);
        for i in 0..d {
            let juror: T::AccountId = account("juror", i, 0);
            <Jurors<T>>::insert(&juror, JurorInfo {
                stake: T::MinJurorStake::get(),
                active_lock: T::MinJurorStake::get(),
            });
            let outcome = winner_outcome.clone();
            let commitment = T::Hashing::hash_of(&(juror.clone(), outcome.clone(), salt));
            let draw =
                Draw {
                    juror,
                    vote: Vote::Revealed { commitment, outcome, salt },
                    weight: 1u32,
                    slashable: T::MinJurorStake::get(),
                };
            draws.try_push(draw).unwrap();
        }
        <Draws<T>>::insert(market_id, draws);
    }: _(RawOrigin::Signed(caller), market_id)

    impl_benchmark_test_suite!(
        Court,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime,
    );
}
