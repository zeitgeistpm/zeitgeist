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

#![cfg(feature = "runtime-benchmarks")]

use crate::{BalanceOf, Call, Config, Event, MarketIdOf, Pallet};
use alloc::{vec, vec::Vec};
use frame_benchmarking::v2::*;
use frame_support::{
    dispatch::RawOrigin,
    traits::{Bounded, Get},
};
use frame_system::Pallet as System;
use orml_traits::MultiCurrency;
use sp_runtime::{traits::Zero, Perbill, SaturatedConversion};
use zeitgeist_primitives::{
    constants::base_multiples::*,
    math::fixed::{BaseProvider, ZeitgeistBase},
    traits::{CombinatorialTokensBenchmarkHelper, MarketCommonsPalletApi},
    types::{Asset, Market, MarketCreation, MarketPeriod, MarketStatus, MarketType, ScoringRule},
};

fn create_market<T: Config>(caller: T::AccountId, asset_count: u16) -> MarketIdOf<T> {
    let market = Market {
        market_id: Default::default(),
        base_asset: Asset::Ztg,
        creation: MarketCreation::Permissionless,
        creator_fee: Perbill::zero(),
        creator: caller.clone(),
        oracle: caller,
        metadata: Default::default(),
        market_type: MarketType::Categorical(asset_count),
        period: MarketPeriod::Block(0u32.into()..1u32.into()),
        deadlines: Default::default(),
        scoring_rule: ScoringRule::AmmCdaHybrid,
        status: MarketStatus::Active,
        report: None,
        resolved_outcome: None,
        dispute_mechanism: None,
        bonds: Default::default(),
        early_close: None,
    };
    T::MarketCommons::push_market(market).unwrap()
}

fn create_payout_vector<T: Config>(asset_count: u16) -> Vec<BalanceOf<T>> {
    let mut result = vec![Zero::zero(); asset_count as usize];
    result[0] = ZeitgeistBase::get().unwrap();

    result
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn redeem_position_sans_parent(n: Linear<2, 128>) {
        let alice: T::AccountId = whitelisted_caller();

        let n_u16: u16 = n.try_into().unwrap();
        let asset_count = n_u16 + 1;

        // `index_set` has `n` entries that are `true`, which results in `n` iterations in the `for`
        // loop in `redeem_position`.
        let mut index_set = vec![true; asset_count as usize];
        *index_set.last_mut().unwrap() = false;

        let parent_collection_id = None;
        let market_id = create_market::<T>(alice.clone(), asset_count);

        let payout_vector = create_payout_vector::<T>(asset_count);
        T::BenchmarkHelper::setup_payout_vector(market_id, Some(payout_vector)).unwrap();

        // Deposit tokens for Alice and the pallet account.
        let position = Pallet::<T>::position_from_parent_collection(
            parent_collection_id,
            market_id,
            index_set.clone(),
            false,
        )
        .unwrap();
        let amount = ZeitgeistBase::get().unwrap();
        T::MultiCurrency::deposit(position, &alice, amount).unwrap();
        T::MultiCurrency::deposit(Asset::Ztg, &Pallet::<T>::account_id(), amount).unwrap();

        #[extrinsic_call]
        redeem_position(
            RawOrigin::Signed(alice.clone()),
            parent_collection_id,
            market_id,
            index_set.clone(),
            true,
        );

        let expected_event = <T as Config>::RuntimeEvent::from(Event::<T>::TokenRedeemed {
            who: alice,
            parent_collection_id,
            market_id,
            index_set,
            asset_in: position,
            amount_in: amount,
            asset_out: Asset::Ztg,
            amount_out: amount,
        });
        System::<T>::assert_last_event(expected_event.into());
    }

    #[benchmark]
    fn redeem_position_with_parent(n: Linear<2, 128>) {
        let alice: T::AccountId = whitelisted_caller();

        let n_u16: u16 = n.try_into().unwrap();
        let asset_count = n_u16 + 1;

        // `index_set` has `n` entries that are `true`, which results in `n` iterations in the `for`
        // loop in `redeem_position`.
        let mut index_set = vec![true; asset_count as usize];
        *index_set.last_mut().unwrap() = false;

        let parent_market_id = create_market::<T>(alice.clone(), 2);
        let cid_01 = Pallet::<T>::collection_id_from_parent_collection(
            None,
            parent_market_id,
            vec![false, true],
            false,
        ).unwrap();
        let pos_01 = Pallet::<T>::position_from_collection_id(parent_market_id, cid_01).unwrap();

        let child_market_id = create_market::<T>(alice.clone(), asset_count);
        let pos_01_10 = Pallet::<T>::position_from_parent_collection(
            Some(cid_01),
            child_market_id,
            index_set.clone(),
            false,
        )
        .unwrap();
        let amount = ZeitgeistBase::get().unwrap();
        T::MultiCurrency::deposit(pos_01_10, &alice, amount).unwrap();

        let payout_vector = create_payout_vector::<T>(asset_count);
        T::BenchmarkHelper::setup_payout_vector(parent_market_id, Some(payout_vector)).unwrap();

        #[extrinsic_call]
        redeem_position(
            RawOrigin::Signed(alice.clone()),
            Some(cid_01),
            child_market_id,
            index_set.clone(),
            true,
        );

        let expected_event = <T as Config>::RuntimeEvent::from(Event::<T>::TokenRedeemed {
            who: alice,
            parent_collection_id: Some(cid_01),
            market_id: child_market_id,
            index_set,
            asset_in: pos_01_10,
            amount_in: amount,
            asset_out: pos_01,
            amount_out: amount,
        });
        System::<T>::assert_last_event(expected_event.into());
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::ext_builder::ExtBuilder::build(),
        crate::mock::runtime::Runtime
    );
}
