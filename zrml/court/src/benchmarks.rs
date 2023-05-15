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
    types::{CourtParticipantInfo, CourtPoolItem, CourtStatus, Draw, Vote},
    AppealInfo, BalanceOf, Call, Config, CourtId, CourtPool, Courts, DelegatedStakesOf,
    MarketIdToCourtId, MarketOf, Pallet as Court, Pallet, Participants, RequestBlock,
    SelectedDraws, VoteItem,
};
use alloc::{vec, vec::Vec};
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::{Currency, Get, NamedReservableCurrency};
use frame_system::RawOrigin;
use sp_arithmetic::Perbill;
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
    let mut pool = <CourtPool<T>>::get();
    let min_amount = T::MinJurorStake::get();
    let max_amount = min_amount + min_amount + BalanceOf::<T>::from(number);
    for i in 0..number {
        let juror: T::AccountId = account("juror", i, 0);
        let stake = max_amount - BalanceOf::<T>::from(i);
        let _ = T::Currency::deposit_creating(&juror, stake);
        <Participants<T>>::insert(
            juror.clone(),
            CourtParticipantInfo {
                stake,
                active_lock: <BalanceOf<T>>::zero(),
                prepare_exit_at: None,
                delegations: Default::default(),
            },
        );
        let consumed_stake = BalanceOf::<T>::zero();
        let pool_item = CourtPoolItem { stake, court_participant: juror.clone(), consumed_stake };
        match pool.binary_search_by_key(&(stake, &juror), |pool_item| {
            (pool_item.stake, &pool_item.court_participant)
        }) {
            Ok(_) => panic!("Juror already in pool"),
            Err(index) => pool.try_insert(index, pool_item).unwrap(),
        };
    }
    <CourtPool<T>>::put(pool);
    Ok(())
}

// assume always worst case for delegations (MaxDelegations),
// because delegations are individual to each juror
fn fill_delegations<T>()
where
    T: Config,
{
    let pool = <CourtPool<T>>::get();
    debug_assert!(pool.len() >= T::MaxDelegations::get() as usize);
    let mut pool_iter = pool.iter();
    let mut delegated_jurors = vec![];
    for _ in 0..T::MaxDelegations::get() {
        let delegated_juror = pool_iter.next().unwrap().court_participant.clone();
        delegated_jurors.push(delegated_juror);
    }
    for pool_item in pool_iter {
        let juror = &pool_item.court_participant;
        let mut j = <Participants<T>>::get(juror).unwrap();
        j.delegations = Some(delegated_jurors.clone().try_into().unwrap());
        <Participants<T>>::insert(juror, j);
    }
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

fn setup_court<T>() -> Result<(crate::MarketIdOf<T>, CourtId), &'static str>
where
    T: Config,
{
    <frame_system::Pallet<T>>::set_block_number(1u64.saturated_into::<T::BlockNumber>());

    let now = <frame_system::Pallet<T>>::block_number();
    <RequestBlock<T>>::put(now + 1u64.saturated_into::<T::BlockNumber>());

    let market_id = T::MarketCommons::push_market(get_market::<T>()).unwrap();
    Court::<T>::on_dispute(&market_id, &get_market::<T>()).unwrap();

    let court_id = <MarketIdToCourtId<T>>::get(market_id).unwrap();

    Ok((market_id, court_id))
}

fn fill_draws<T>(court_id: CourtId, number: u32) -> Result<(), &'static str>
where
    T: Config,
{
    // remove last random selections of on_dispute
    <SelectedDraws<T>>::remove(court_id);
    let mut draws = <SelectedDraws<T>>::get(court_id);
    for i in 0..number {
        let juror = account("juror", i, 0);
        deposit::<T>(&juror);
        <Participants<T>>::insert(
            &juror,
            CourtParticipantInfo {
                stake: T::MinJurorStake::get(),
                active_lock: T::MinJurorStake::get(),
                prepare_exit_at: None,
                delegations: Default::default(),
            },
        );
        let draw = Draw {
            court_participant: juror,
            vote: Vote::Drawn,
            weight: 1u32,
            slashable: T::MinJurorStake::get(),
        };
        let index = draws
            .binary_search_by_key(&draw.court_participant, |draw| draw.court_participant.clone())
            .unwrap_or_else(|j| j);
        draws.try_insert(index, draw).unwrap();
    }
    <SelectedDraws<T>>::insert(court_id, draws);
    Ok(())
}

fn apply_revealed_draws<T>(court_id: CourtId)
where
    T: Config,
{
    let winner_outcome = OutcomeReport::Scalar(0u128);
    let mut draws = <SelectedDraws<T>>::get(court_id);
    // change draws to have revealed votes
    for draw in draws.iter_mut() {
        let salt = Default::default();
        let commitment =
            T::Hashing::hash_of(&(draw.court_participant.clone(), winner_outcome.clone(), salt));
        draw.vote = Vote::Revealed {
            commitment,
            vote_item: VoteItem::Outcome(winner_outcome.clone()),
            salt,
        };
    }
    <SelectedDraws<T>>::insert(court_id, draws);
}

benchmarks! {
    join_court {
        let j in 0..(T::MaxCourtParticipants::get() - 1);

        fill_pool::<T>(j)?;

        let caller: T::AccountId = whitelisted_caller();
        join_with_min_stake::<T>(&caller)?;

        let new_stake = T::MinJurorStake::get()
            .saturating_add(1u128.saturated_into::<BalanceOf<T>>());
    }: _(RawOrigin::Signed(caller), new_stake)

    delegate {
        // jurors greater or equal to MaxDelegations,
        // because we can not delegate to a non-existent juror
        let j in 5..(T::MaxCourtParticipants::get() - 1);
        let d in 1..T::MaxDelegations::get();

        fill_pool::<T>(j)?;

        let caller: T::AccountId = whitelisted_caller();
        join_with_min_stake::<T>(&caller)?;

        let juror_pool = <CourtPool<T>>::get();
        let mut delegations = Vec::<T::AccountId>::new();
        juror_pool.iter()
            .filter(|pool_item| pool_item.court_participant != caller).take(d as usize)
            .for_each(|pool_item| delegations.push(pool_item.court_participant.clone()));

        let new_stake = T::MinJurorStake::get()
            .saturating_add(1u128.saturated_into::<BalanceOf<T>>());
    }: _(RawOrigin::Signed(caller), new_stake, delegations)

    prepare_exit_court {
        let j in 0..(T::MaxCourtParticipants::get() - 1);

        fill_pool::<T>(j)?;

        let caller: T::AccountId = whitelisted_caller();
        join_with_min_stake::<T>(&caller)?;
    }: _(RawOrigin::Signed(caller))

    exit_court_remove {
        let caller: T::AccountId = whitelisted_caller();
        join_with_min_stake::<T>(&caller)?;

        Court::<T>::prepare_exit_court(RawOrigin::Signed(caller.clone()).into())?;
        let now = <frame_system::Pallet<T>>::block_number();
        <frame_system::Pallet<T>>::set_block_number(now + T::InflationPeriod::get());

        <Participants<T>>::mutate(caller.clone(), |prev_p_info| {
            prev_p_info.as_mut().unwrap().active_lock = <BalanceOf<T>>::zero();
        });

        let juror = T::Lookup::unlookup(caller.clone());
    }: exit_court(RawOrigin::Signed(caller), juror)

    exit_court_set {
        let caller: T::AccountId = whitelisted_caller();
        join_with_min_stake::<T>(&caller)?;

        Court::<T>::prepare_exit_court(RawOrigin::Signed(caller.clone()).into())?;
        let now = <frame_system::Pallet<T>>::block_number();
        <frame_system::Pallet<T>>::set_block_number(now + T::InflationPeriod::get());

        <Participants<T>>::mutate(caller.clone(), |prev_p_info| {
            prev_p_info.as_mut().unwrap().active_lock = T::MinJurorStake::get();
        });

        let juror = T::Lookup::unlookup(caller.clone());
    }: exit_court(RawOrigin::Signed(caller), juror)

    vote {
        let d in 1..T::MaxSelectedDraws::get();

        fill_pool::<T>(T::MaxCourtParticipants::get() - 1)?;

        let caller: T::AccountId = whitelisted_caller();
        let (market_id, court_id) = setup_court::<T>()?;

        let court = <Courts<T>>::get(court_id).unwrap();
        let pre_vote = court.cycle_ends.pre_vote;

        fill_draws::<T>(court_id, d)?;

        let mut draws = <SelectedDraws<T>>::get(court_id);
        let draws_len = draws.len();
        draws.remove(0);
        let draw = Draw {
            court_participant: caller.clone(),
            vote: Vote::Drawn,
            weight: 1u32,
            slashable: <BalanceOf<T>>::zero(),
        };
        let index = draws.binary_search_by_key(&caller, |draw| draw.court_participant.clone()).unwrap_or_else(|j| j);
        draws.try_insert(index, draw).unwrap();
        <SelectedDraws<T>>::insert(court_id, draws);

        <frame_system::Pallet<T>>::set_block_number(pre_vote + 1u64.saturated_into::<T::BlockNumber>());

        let commitment_vote = Default::default();
    }: _(RawOrigin::Signed(caller), court_id, commitment_vote)

    denounce_vote {
        let d in 1..T::MaxSelectedDraws::get();

        let necessary_draws_weight: usize = Court::<T>::necessary_draws_weight(0usize);
        fill_pool::<T>(necessary_draws_weight as u32)?;

        let caller: T::AccountId = whitelisted_caller();
        let (market_id, court_id) = setup_court::<T>()?;

        let court = <Courts<T>>::get(court_id).unwrap();
        let pre_vote = court.cycle_ends.pre_vote;

        fill_draws::<T>(court_id, d)?;

        let salt = Default::default();
        let outcome = OutcomeReport::Scalar(0u128);
        let vote_item = VoteItem::Outcome(outcome);
        let denounced_juror: T::AccountId = account("denounced_juror", 0, 0);
        join_with_min_stake::<T>(&denounced_juror)?;
        <Participants<T>>::insert(&denounced_juror, CourtParticipantInfo {
            stake: T::MinJurorStake::get(),
            active_lock: T::MinJurorStake::get(),
            prepare_exit_at: None,
            delegations: Default::default(),
        });
        let denounced_juror_unlookup = T::Lookup::unlookup(denounced_juror.clone());
        let commitment = T::Hashing::hash_of(&(denounced_juror.clone(), vote_item.clone(), salt));

        let mut draws = <SelectedDraws<T>>::get(court_id);
        draws.remove(0);
        let draws_len = draws.len();
        let index = draws.binary_search_by_key(&denounced_juror, |draw| draw.court_participant.clone()).unwrap_or_else(|j| j);
        let draw = Draw {
            court_participant: denounced_juror,
            vote: Vote::Secret { commitment },
            weight: 1u32,
            slashable: T::MinJurorStake::get(),
        };
        draws.try_insert(index, draw).unwrap();
        <SelectedDraws<T>>::insert(court_id, draws);

        <frame_system::Pallet<T>>::set_block_number(pre_vote + 1u64.saturated_into::<T::BlockNumber>());
    }: _(RawOrigin::Signed(caller), court_id, denounced_juror_unlookup, vote_item, salt)

    reveal_vote {
        let d in 1..T::MaxSelectedDraws::get();

        fill_pool::<T>(T::MaxCourtParticipants::get() - 1)?;

        let caller: T::AccountId = whitelisted_caller();
        let (market_id, court_id) = setup_court::<T>()?;

        let court = <Courts<T>>::get(court_id).unwrap();
        let vote_end = court.cycle_ends.vote;

        fill_draws::<T>(court_id, d)?;

        let salt = Default::default();
        let outcome = OutcomeReport::Scalar(0u128);
        let vote_item = VoteItem::Outcome(outcome);
        join_with_min_stake::<T>(&caller)?;
        <Participants<T>>::insert(&caller, CourtParticipantInfo {
            stake: T::MinJurorStake::get(),
            active_lock: T::MinJurorStake::get(),
            prepare_exit_at: None,
            delegations: Default::default(),
        });
        let commitment = T::Hashing::hash_of(&(caller.clone(), vote_item.clone(), salt));

        let mut draws = <SelectedDraws<T>>::get(court_id);
        let draws_len = draws.len();
        draws.remove(0);
        let index = draws.binary_search_by_key(&caller, |draw| draw.court_participant.clone()).unwrap_or_else(|j| j);
        draws.try_insert(index, Draw {
            court_participant: caller.clone(),
            vote: Vote::Secret { commitment },
            weight: 1u32,
            slashable: T::MinJurorStake::get(),
        }).unwrap();
        <SelectedDraws<T>>::insert(court_id, draws);

        <frame_system::Pallet<T>>::set_block_number(vote_end + 1u64.saturated_into::<T::BlockNumber>());
    }: _(RawOrigin::Signed(caller), court_id, vote_item, salt)

    appeal {
        // from 255 because in the last appeal round we need at least 255 jurors
        let j in 255..T::MaxCourtParticipants::get();
        let a in 0..(T::MaxAppeals::get() - 2);
        let r in 0..62;
        let f in 0..62;

        let necessary_draws_weight = Court::<T>::necessary_draws_weight((T::MaxAppeals::get() - 1) as usize);
        debug_assert!(necessary_draws_weight == 255usize);
        fill_pool::<T>(j)?;
        fill_delegations::<T>();

        let caller: T::AccountId = whitelisted_caller();
        deposit::<T>(&caller);
        let (market_id, court_id) = setup_court::<T>()?;

        let mut court = <Courts<T>>::get(court_id).unwrap();
        let appeal_end = court.cycle_ends.appeal;
        for i in 0..r {
            let market_id_i = (i + 100).saturated_into::<crate::MarketIdOf<T>>();
            T::DisputeResolution::add_auto_resolve(&market_id_i, appeal_end).unwrap();
        }
        T::DisputeResolution::add_auto_resolve(&market_id, appeal_end).unwrap();

        let aggregation = court.cycle_ends.aggregation;
        for i in 0..a {
            let appeal_info = AppealInfo {
                backer: account("backer", i, 0),
                bond: crate::get_appeal_bond::<T>(i as usize),
                appealed_vote_item: VoteItem::Outcome(OutcomeReport::Scalar(0u128)),
            };
            court.appeals.try_push(appeal_info).unwrap();
        }
        <Courts<T>>::insert(court_id, court);

        let salt = Default::default();
        // remove last random selections of on_dispute
        <SelectedDraws<T>>::remove(court_id);
        let mut draws = <SelectedDraws<T>>::get(court_id);
        let draws_len = Court::<T>::necessary_draws_weight(a as usize) as u32;
        for i in 0..draws_len {
            let juror: T::AccountId = account("juror", i, 0);
            <Participants<T>>::insert(&juror, CourtParticipantInfo {
                stake: T::MinJurorStake::get(),
                active_lock: T::MinJurorStake::get(),
                prepare_exit_at: None,
                delegations: Default::default(),
            });
            let vote_item: VoteItem = VoteItem::Outcome(OutcomeReport::Scalar(i as u128));
            let commitment = T::Hashing::hash_of(&(juror.clone(), vote_item.clone(), salt));
            let draw =
                Draw {
                    court_participant: juror,
                    vote: Vote::Revealed { commitment, vote_item, salt },
                    weight: 1u32,
                    slashable: <BalanceOf<T>>::zero()
                };
            draws.try_push(draw).unwrap();
        }
        <SelectedDraws<T>>::insert(court_id, draws);

        <frame_system::Pallet<T>>::set_block_number(aggregation + 1u64.saturated_into::<T::BlockNumber>());
        let now = <frame_system::Pallet<T>>::block_number();
        <RequestBlock<T>>::put(now + 1u64.saturated_into::<T::BlockNumber>());

        let new_resolve_at = <RequestBlock<T>>::get()
            + T::VotePeriod::get()
            + T::AggregationPeriod::get()
            + T::AppealPeriod::get();
        for i in 0..f {
            let market_id_i = (i + 100).saturated_into::<crate::MarketIdOf<T>>();
            T::DisputeResolution::add_auto_resolve(&market_id_i, new_resolve_at).unwrap();
        }
    }: _(RawOrigin::Signed(caller), court_id)
    verify {
        let court = <Courts<T>>::get(court_id).unwrap();
        assert_eq!(court.cycle_ends.appeal, new_resolve_at);
    }

    reassign_court_stakes {
        // because we have 5 MaxDelegations
        let d in 5..T::MaxSelectedDraws::get();
        debug_assert!(T::MaxDelegations::get() < T::MaxSelectedDraws::get());

        // just to initialize the court
        let necessary_draws_weight: usize = Court::<T>::necessary_draws_weight(0usize);
        fill_pool::<T>(necessary_draws_weight as u32)?;

        let caller: T::AccountId = whitelisted_caller();
        let (market_id, court_id) = setup_court::<T>()?;

        let mut court = <Courts<T>>::get(court_id).unwrap();
        let winner_outcome = OutcomeReport::Scalar(0u128);
        let wrong_outcome = OutcomeReport::Scalar(1u128);
        let winner_vote_item = VoteItem::Outcome(winner_outcome);
        let wrong_vote_item = VoteItem::Outcome(wrong_outcome);
        court.status = CourtStatus::Closed { winner: winner_vote_item.clone() };
        <Courts<T>>::insert(court_id, court);

        let salt = Default::default();
        // remove last random selections of on_dispute
        <SelectedDraws<T>>::remove(court_id);
        let mut draws = <SelectedDraws<T>>::get(court_id);
        let mut delegated_stakes: DelegatedStakesOf<T> = Default::default();
        for i in 0..d {
            let juror: T::AccountId = account("juror_i", i, 0);
            deposit::<T>(&juror);
            <Participants<T>>::insert(&juror, CourtParticipantInfo {
                stake: T::MinJurorStake::get(),
                active_lock: T::MinJurorStake::get(),
                prepare_exit_at: None,
                delegations: Default::default(),
            });
            let draw = if i < T::MaxDelegations::get() {
                delegated_stakes.try_push((juror.clone(), T::MinJurorStake::get())).unwrap();

                let vote_item: VoteItem = if i % 2 == 0 {
                    wrong_vote_item.clone()
                } else {
                    winner_vote_item.clone()
                };
                let commitment = T::Hashing::hash_of(&(juror.clone(), vote_item.clone(), salt));
                Draw {
                    court_participant: juror,
                    vote: Vote::Revealed { commitment, vote_item, salt },
                    weight: 1u32,
                    slashable: T::MinJurorStake::get(),
                }
            } else {
                Draw {
                    court_participant: juror,
                    vote: Vote::Delegated { delegated_stakes: delegated_stakes.clone() },
                    weight: 1u32,
                    slashable: T::MinJurorStake::get(),
                }
            };
            draws.try_push(draw).unwrap();
        }
        <SelectedDraws<T>>::insert(court_id, draws);
    }: _(RawOrigin::Signed(caller), court_id)

    set_inflation {
        let inflation = Perbill::from_percent(10);
    }: _(RawOrigin::Root, inflation)

    handle_inflation {
        let j in 1..T::MaxCourtParticipants::get();
        fill_pool::<T>(j)?;

        <frame_system::Pallet<T>>::set_block_number(T::InflationPeriod::get());
        let now = <frame_system::Pallet<T>>::block_number();
    }: {
        Court::<T>::handle_inflation(now);
    }

    select_jurors {
        let a in 0..(T::MaxAppeals::get() - 1);
        fill_pool::<T>(T::MaxCourtParticipants::get())?;

        fill_delegations::<T>();
    }: {
        let _ = Court::<T>::select_jurors(a as usize).unwrap();
    }

    on_dispute {
        let j in 31..T::MaxCourtParticipants::get();
        let r in 0..62;

        let now = <frame_system::Pallet<T>>::block_number();
        let pre_vote_end = now + 1u64.saturated_into::<T::BlockNumber>();
        <RequestBlock<T>>::put(pre_vote_end);

        let appeal_end = pre_vote_end
            + T::VotePeriod::get()
            + T::AggregationPeriod::get()
            + T::AppealPeriod::get();

        for i in 0..r {
            let market_id_i = (i + 100).saturated_into::<crate::MarketIdOf<T>>();
            T::DisputeResolution::add_auto_resolve(&market_id_i, appeal_end).unwrap();
        }

        fill_pool::<T>(j)?;

        let market_id = 0u32.into();
        let market = get_market::<T>();
        T::MarketCommons::push_market(market.clone()).unwrap();
    }: {
        Court::<T>::on_dispute(&market_id, &market).unwrap();
    }

    on_resolution {
        let d in 1..T::MaxSelectedDraws::get();

        let necessary_draws_weight: usize = Court::<T>::necessary_draws_weight(0usize);
        fill_pool::<T>(necessary_draws_weight as u32)?;

        let (market_id, court_id) = setup_court::<T>()?;
        let market = get_market::<T>();

        fill_draws::<T>(court_id, d)?;

        let winner_outcome = OutcomeReport::Scalar(0u128);
        let mut draws = <SelectedDraws<T>>::get(court_id);
        // change draws to have revealed votes
        for draw in draws.iter_mut() {
            let salt = Default::default();
            let commitment = T::Hashing::hash_of(&(draw.court_participant.clone(), winner_outcome.clone(), salt));
            draw.vote = Vote::Revealed {
                commitment,
                vote_item: VoteItem::Outcome(winner_outcome.clone()),
                salt,
            };
        }
        <SelectedDraws<T>>::insert(court_id, draws);
    }: {
        Court::<T>::on_resolution(&market_id, &market).unwrap();
    }

    exchange {
        let a in 0..T::MaxAppeals::get();

        let necessary_draws_weight: usize = Court::<T>::necessary_draws_weight(0usize);
        fill_pool::<T>(necessary_draws_weight as u32)?;
        let (market_id, court_id) = setup_court::<T>()?;
        let market = get_market::<T>();

        let mut court = <Courts<T>>::get(court_id).unwrap();

        let resolved_outcome = OutcomeReport::Scalar(0u128);
        for i in 0..a {
            let backer = account("backer", i, 0);
            let bond = T::MinJurorStake::get();
            let _ = T::Currency::deposit_creating(&backer, bond);
            T::Currency::reserve_named(&Court::<T>::reserve_id(), &backer, bond).unwrap();
            let appeal_info = AppealInfo {
                backer,
                bond,
                appealed_vote_item: VoteItem::Outcome(resolved_outcome.clone()),
            };
            court.appeals.try_push(appeal_info).unwrap();
        }
        <Courts<T>>::insert(court_id, court);
    }: {
        Court::<T>::exchange(&market_id, &market, &resolved_outcome, Default::default()).unwrap();
    }

    get_auto_resolve {
        let necessary_draws_weight: usize = Court::<T>::necessary_draws_weight(0usize);
        fill_pool::<T>(necessary_draws_weight as u32)?;
        let (market_id, court_id) = setup_court::<T>()?;
        let market = get_market::<T>();
    }: {
        Court::<T>::get_auto_resolve(&market_id, &market);
    }

    has_failed {
        let necessary_draws_weight: usize = Court::<T>::necessary_draws_weight(0usize);
        fill_pool::<T>(necessary_draws_weight as u32)?;
        let (market_id, court_id) = setup_court::<T>()?;
        let market = get_market::<T>();
    }: {
        Court::<T>::has_failed(&market_id, &market).unwrap();
    }

    on_global_dispute {
        let a in 0..T::MaxAppeals::get();
        let d in 1..T::MaxSelectedDraws::get();

        let necessary_draws_weight: usize = Court::<T>::necessary_draws_weight(0usize);
        fill_pool::<T>(necessary_draws_weight as u32)?;
        let (market_id, court_id) = setup_court::<T>()?;
        let market = get_market::<T>();

        fill_draws::<T>(court_id, d)?;
        apply_revealed_draws::<T>(court_id);

        let resolved_outcome = OutcomeReport::Scalar(0u128);

        let mut court = <Courts<T>>::get(court_id).unwrap();
        for i in 0..a {
            let backer = account("backer", i, 0);
            let bond = T::MinJurorStake::get();
            let _ = T::Currency::deposit_creating(&backer, bond);
            T::Currency::reserve_named(&Court::<T>::reserve_id(), &backer, bond).unwrap();
            let appeal_info = AppealInfo {
                backer,
                bond,
                appealed_vote_item: VoteItem::Outcome(resolved_outcome.clone()),
            };
            court.appeals.try_push(appeal_info).unwrap();
        }
        <Courts<T>>::insert(court_id, court);
    }: {
        Court::<T>::on_global_dispute(&market_id, &market).unwrap();
    }

    clear {
        let d in 1..T::MaxSelectedDraws::get();

        let necessary_draws_weight: usize = Court::<T>::necessary_draws_weight(0usize);
        fill_pool::<T>(necessary_draws_weight as u32)?;

        let (market_id, court_id) = setup_court::<T>()?;
        let market = get_market::<T>();

        fill_draws::<T>(court_id, d)?;
        apply_revealed_draws::<T>(court_id);
    }: {
        Court::<T>::clear(&market_id, &market).unwrap();
    }

    impl_benchmark_test_suite!(
        Court,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime,
    );
}
