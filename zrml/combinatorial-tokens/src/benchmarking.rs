// Copyright 2024-2025 Forecasting Technologies LTD.
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
use frame_support::dispatch::RawOrigin;
use frame_system::Pallet as System;
use orml_traits::MultiCurrency;
use sp_runtime::{traits::Zero, Perbill};
use zeitgeist_primitives::{
    math::fixed::{BaseProvider, ZeitgeistBase},
    traits::{CombinatorialTokensBenchmarkHelper, CombinatorialTokensFuel, MarketCommonsPalletApi},
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
    fn split_position_vertical_sans_parent(n: Linear<2, 32>, m: Linear<32, 64>) {
        let alice: T::AccountId = whitelisted_caller();

        let position_count: usize = n.try_into().unwrap();
        let total = m;

        let parent_collection_id = None;
        let market_id = create_market::<T>(alice.clone(), position_count.try_into().unwrap());
        // Partition is 10...0, 010...0, ..., 0...01.
        let partition: Vec<_> = (0..position_count)
            .map(|index| {
                let mut index_set = vec![false; position_count];
                index_set[index] = true;

                index_set
            })
            .collect();
        let amount = ZeitgeistBase::get().unwrap();

        T::MultiCurrency::deposit(Asset::Ztg, &alice, amount).unwrap();

        #[extrinsic_call]
        split_position(
            RawOrigin::Signed(alice.clone()),
            parent_collection_id,
            market_id,
            partition.clone(),
            amount,
            T::Fuel::from_total(total),
        );

        let collection_ids: Vec<_> = partition
            .iter()
            .cloned()
            .map(|index_set| {
                Pallet::<T>::collection_id_from_parent_collection(
                    parent_collection_id,
                    market_id,
                    index_set,
                    T::Fuel::from_total(total),
                )
                .unwrap()
            })
            .collect();
        let assets_out: Vec<_> = collection_ids
            .iter()
            .cloned()
            .map(|collection_id| {
                Pallet::<T>::position_from_collection_id(market_id, collection_id).unwrap()
            })
            .collect();
        let expected_event = <T as Config>::RuntimeEvent::from(Event::<T>::TokenSplit {
            who: alice,
            parent_collection_id,
            market_id,
            partition,
            asset_in: Asset::Ztg,
            assets_out,
            collection_ids,
            amount,
        });
        System::<T>::assert_last_event(expected_event.into());
    }

    #[benchmark]
    fn split_position_vertical_with_parent(n: Linear<2, 32>, m: Linear<32, 64>) {
        let alice: T::AccountId = whitelisted_caller();

        let position_count: usize = n.try_into().unwrap();
        let total = m;

        let parent_collection_id = None;
        let parent_market_id = create_market::<T>(alice.clone(), 2);

        // The collection/position that we're merging into.
        let cid_01 = Pallet::<T>::collection_id_from_parent_collection(
            parent_collection_id,
            parent_market_id,
            vec![false, true],
            T::Fuel::from_total(total),
        )
        .unwrap();
        let pos_01 = Pallet::<T>::position_from_collection_id(parent_market_id, cid_01).unwrap();

        let child_market_id = create_market::<T>(alice.clone(), position_count.try_into().unwrap());
        let partition: Vec<_> = (0..position_count)
            .map(|index| {
                let mut index_set = vec![false; position_count];
                index_set[index] = true;

                index_set
            })
            .collect();
        let amount = ZeitgeistBase::get().unwrap();

        T::MultiCurrency::deposit(pos_01, &alice, amount).unwrap();

        #[extrinsic_call]
        split_position(
            RawOrigin::Signed(alice.clone()),
            Some(cid_01),
            child_market_id,
            partition.clone(),
            amount,
            T::Fuel::from_total(total),
        );

        let collection_ids: Vec<_> = partition
            .iter()
            .cloned()
            .map(|index_set| {
                Pallet::<T>::collection_id_from_parent_collection(
                    Some(cid_01),
                    child_market_id,
                    index_set,
                    T::Fuel::from_total(total),
                )
                .unwrap()
            })
            .collect();
        let assets_out: Vec<_> = collection_ids
            .iter()
            .cloned()
            .map(|collection_id| {
                Pallet::<T>::position_from_collection_id(child_market_id, collection_id).unwrap()
            })
            .collect();
        let expected_event = <T as Config>::RuntimeEvent::from(Event::<T>::TokenSplit {
            who: alice,
            parent_collection_id: Some(cid_01),
            market_id: child_market_id,
            partition,
            asset_in: pos_01,
            assets_out,
            collection_ids,
            amount,
        });
        System::<T>::assert_last_event(expected_event.into());
    }

    #[benchmark]
    fn split_position_horizontal(n: Linear<2, 32>, m: Linear<32, 64>) {
        let alice: T::AccountId = whitelisted_caller();

        let position_count: usize = n.try_into().unwrap();
        let asset_count = position_count + 1;
        let total = m;

        let parent_collection_id = None;
        let market_id = create_market::<T>(alice.clone(), asset_count.try_into().unwrap());
        // Partition is 10...0, 010...0, ..., 0...010. Doesn't contain 0...01.
        let partition: Vec<_> = (0..position_count)
            .map(|index| {
                let mut index_set = vec![false; asset_count];
                index_set[index] = true;

                index_set
            })
            .collect();
        let amount = ZeitgeistBase::get().unwrap();

        // Add 1...10 to Alice's account.
        let mut asset_in_index_set = vec![true; asset_count];
        *asset_in_index_set.last_mut().unwrap() = false;
        let asset_in = Pallet::<T>::position_from_parent_collection(
            parent_collection_id,
            market_id,
            asset_in_index_set,
            T::Fuel::from_total(total),
        )
        .unwrap();
        T::MultiCurrency::deposit(asset_in, &alice, amount).unwrap();

        #[extrinsic_call]
        split_position(
            RawOrigin::Signed(alice.clone()),
            parent_collection_id,
            market_id,
            partition.clone(),
            amount,
            T::Fuel::from_total(total),
        );

        let collection_ids: Vec<_> = partition
            .iter()
            .cloned()
            .map(|index_set| {
                Pallet::<T>::collection_id_from_parent_collection(
                    parent_collection_id,
                    market_id,
                    index_set,
                    T::Fuel::from_total(total),
                )
                .unwrap()
            })
            .collect();
        let assets_out: Vec<_> = collection_ids
            .iter()
            .cloned()
            .map(|collection_id| {
                Pallet::<T>::position_from_collection_id(market_id, collection_id).unwrap()
            })
            .collect();
        let expected_event = <T as Config>::RuntimeEvent::from(Event::<T>::TokenSplit {
            who: alice,
            parent_collection_id,
            market_id,
            partition,
            asset_in,
            assets_out,
            collection_ids,
            amount,
        });
        System::<T>::assert_last_event(expected_event.into());
    }

    #[benchmark]
    fn merge_position_vertical_sans_parent(n: Linear<2, 32>, m: Linear<32, 64>) {
        let alice: T::AccountId = whitelisted_caller();

        let position_count: usize = n.try_into().unwrap();
        let total = m;

        let parent_collection_id = None;
        let market_id = create_market::<T>(alice.clone(), position_count.try_into().unwrap());
        let partition: Vec<_> = (0..position_count)
            .map(|index| {
                let mut index_set = vec![false; position_count];
                index_set[index] = true;

                index_set
            })
            .collect();
        let amount = ZeitgeistBase::get().unwrap();

        let assets_in: Vec<_> = partition
            .iter()
            .cloned()
            .map(|index_set| {
                Pallet::<T>::position_from_parent_collection(
                    parent_collection_id,
                    market_id,
                    index_set,
                    T::Fuel::from_total(total),
                )
                .unwrap()
            })
            .collect();

        for &asset in assets_in.iter() {
            T::MultiCurrency::deposit(asset, &alice, amount).unwrap();
        }
        T::MultiCurrency::deposit(Asset::Ztg, &Pallet::<T>::account_id(), amount).unwrap();

        #[extrinsic_call]
        merge_position(
            RawOrigin::Signed(alice.clone()),
            parent_collection_id,
            market_id,
            partition.clone(),
            amount,
            T::Fuel::from_total(total),
        );

        let expected_event = <T as Config>::RuntimeEvent::from(Event::<T>::TokenMerged {
            who: alice,
            parent_collection_id,
            market_id,
            partition,
            asset_out: Asset::Ztg,
            assets_in,
            amount,
        });
        System::<T>::assert_last_event(expected_event.into());
    }

    #[benchmark]
    fn merge_position_vertical_with_parent(n: Linear<2, 32>, m: Linear<32, 64>) {
        let alice: T::AccountId = whitelisted_caller();

        let position_count: usize = n.try_into().unwrap();
        let total = m;

        let parent_collection_id = None;
        let parent_market_id = create_market::<T>(alice.clone(), 2);

        // The collection/position that we're merging into.
        let cid_01 = Pallet::<T>::collection_id_from_parent_collection(
            parent_collection_id,
            parent_market_id,
            vec![false, true],
            T::Fuel::from_total(total),
        )
        .unwrap();
        let pos_01 = Pallet::<T>::position_from_collection_id(parent_market_id, cid_01).unwrap();

        let child_market_id = create_market::<T>(alice.clone(), position_count.try_into().unwrap());
        let partition: Vec<_> = (0..position_count)
            .map(|index| {
                let mut index_set = vec![false; position_count];
                index_set[index] = true;

                index_set
            })
            .collect();
        let amount = ZeitgeistBase::get().unwrap();

        let assets_in: Vec<_> = partition
            .iter()
            .cloned()
            .map(|index_set| {
                Pallet::<T>::position_from_parent_collection(
                    Some(cid_01),
                    child_market_id,
                    index_set,
                    T::Fuel::from_total(total),
                )
                .unwrap()
            })
            .collect();

        for &asset in assets_in.iter() {
            T::MultiCurrency::deposit(asset, &alice, amount).unwrap();
        }

        #[extrinsic_call]
        merge_position(
            RawOrigin::Signed(alice.clone()),
            Some(cid_01),
            child_market_id,
            partition.clone(),
            amount,
            T::Fuel::from_total(total),
        );

        let expected_event = <T as Config>::RuntimeEvent::from(Event::<T>::TokenMerged {
            who: alice,
            parent_collection_id: Some(cid_01),
            market_id: child_market_id,
            partition,
            asset_out: pos_01,
            assets_in,
            amount,
        });
        System::<T>::assert_last_event(expected_event.into());
    }

    #[benchmark]
    fn merge_position_horizontal(n: Linear<2, 32>, m: Linear<32, 64>) {
        let alice: T::AccountId = whitelisted_caller();

        let position_count: usize = n.try_into().unwrap();
        let asset_count = position_count + 1;
        let total = m;

        let parent_collection_id = None;
        let market_id = create_market::<T>(alice.clone(), asset_count.try_into().unwrap());
        // Partition is 10...0, 010...0, ..., 0...010. Doesn't contain 0...01.
        let partition: Vec<_> = (0..position_count)
            .map(|index| {
                let mut index_set = vec![false; asset_count];
                index_set[index] = true;

                index_set
            })
            .collect();
        let amount = ZeitgeistBase::get().unwrap();

        let assets_in: Vec<_> = partition
            .iter()
            .cloned()
            .map(|index_set| {
                Pallet::<T>::position_from_parent_collection(
                    parent_collection_id,
                    market_id,
                    index_set,
                    T::Fuel::from_total(total),
                )
                .unwrap()
            })
            .collect();

        for &asset in assets_in.iter() {
            T::MultiCurrency::deposit(asset, &alice, amount).unwrap();
        }

        #[extrinsic_call]
        merge_position(
            RawOrigin::Signed(alice.clone()),
            parent_collection_id,
            market_id,
            partition.clone(),
            amount,
            T::Fuel::from_total(total),
        );

        let mut asset_out_index_set = vec![true; asset_count];
        *asset_out_index_set.last_mut().unwrap() = false;
        let asset_out = Pallet::<T>::position_from_parent_collection(
            parent_collection_id,
            market_id,
            asset_out_index_set,
            T::Fuel::from_total(total),
        )
        .unwrap();
        let expected_event = <T as Config>::RuntimeEvent::from(Event::<T>::TokenMerged {
            who: alice,
            parent_collection_id,
            market_id,
            partition,
            asset_out,
            assets_in,
            amount,
        });
        System::<T>::assert_last_event(expected_event.into());
    }

    #[benchmark]
    fn redeem_position_sans_parent(n: Linear<2, 32>, m: Linear<32, 64>) {
        let alice: T::AccountId = whitelisted_caller();

        let n_u16: u16 = n.try_into().unwrap();
        let asset_count = n_u16 + 1;
        let total = m;

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
            T::Fuel::from_total(total),
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
            T::Fuel::from_total(total),
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
    fn redeem_position_with_parent(n: Linear<2, 32>, m: Linear<32, 64>) {
        let alice: T::AccountId = whitelisted_caller();

        let n_u16: u16 = n.try_into().unwrap();
        let asset_count = n_u16 + 1;
        let total = m;

        // `index_set` has `n` entries that are `true`, which results in `n` iterations in the `for`
        // loop in `redeem_position`.
        let mut index_set = vec![true; asset_count as usize];
        *index_set.last_mut().unwrap() = false;

        let parent_market_id = create_market::<T>(alice.clone(), 2);
        let cid_01 = Pallet::<T>::collection_id_from_parent_collection(
            None,
            parent_market_id,
            vec![false, true],
            T::Fuel::from_total(total),
        )
        .unwrap();
        let pos_01 = Pallet::<T>::position_from_collection_id(parent_market_id, cid_01).unwrap();

        let child_market_id = create_market::<T>(alice.clone(), asset_count);
        let pos_01_10 = Pallet::<T>::position_from_parent_collection(
            Some(cid_01),
            child_market_id,
            index_set.clone(),
            T::Fuel::from_total(total),
        )
        .unwrap();
        let amount = ZeitgeistBase::get().unwrap();
        T::MultiCurrency::deposit(pos_01_10, &alice, amount).unwrap();

        let payout_vector = create_payout_vector::<T>(asset_count);
        T::BenchmarkHelper::setup_payout_vector(child_market_id, Some(payout_vector)).unwrap();

        #[extrinsic_call]
        redeem_position(
            RawOrigin::Signed(alice.clone()),
            Some(cid_01),
            child_market_id,
            index_set.clone(),
            T::Fuel::from_total(total),
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
