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

#![cfg(test)]

use crate::{
    mock::{ExtBuilder, MarketCommons, Runtime},
    MarketCounter, Markets,
};
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_runtime::DispatchError;
use zeitgeist_primitives::{
    traits::MarketCommonsPalletApi,
    types::{
        AccountIdTest, Asset, Balance, BlockNumber, Deadlines, Market, MarketBonds, MarketCreation,
        MarketDisputeMechanism, MarketId, MarketPeriod, MarketStatus, MarketType, Moment,
        ScoringRule,
    },
};

const MARKET_DUMMY: Market<AccountIdTest, Balance, BlockNumber, Moment, Asset<MarketId>> = Market {
    base_asset: Asset::Ztg,
    creation: MarketCreation::Permissionless,
    creator_fee: 0,
    creator: 0,
    market_type: MarketType::Scalar(0..=100),
    dispute_mechanism: MarketDisputeMechanism::Authorized,
    metadata: vec![],
    oracle: 0,
    period: MarketPeriod::Block(0..100),
    deadlines: Deadlines { grace_period: 1_u64, oracle_duration: 1_u64, dispute_duration: 1_u64 },
    report: None,
    resolved_outcome: None,
    scoring_rule: ScoringRule::CPMM,
    status: MarketStatus::Disputed,
    bonds: MarketBonds { creation: None, oracle: None, outsider: None },
};

#[test]
fn latest_market_id_interacts_correctly_with_push_market() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(MarketCommons::push_market(market_mock(0)));
        assert_eq!(MarketCommons::latest_market_id().unwrap(), 0);
        assert_ok!(MarketCommons::push_market(market_mock(1)));
        assert_eq!(MarketCommons::latest_market_id().unwrap(), 1);
        assert_ok!(MarketCommons::push_market(market_mock(1)));
        assert_eq!(MarketCommons::latest_market_id().unwrap(), 2);
    });
}

#[test]
fn latest_market_id_fails_if_there_are_no_markets() {
    ExtBuilder::default().build().execute_with(|| {
        assert_err!(
            MarketCommons::latest_market_id(),
            crate::Error::<Runtime>::NoMarketHasBeenCreated
        );
    });
}

#[test]
fn market_interacts_correctly_with_push_market() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(MarketCommons::push_market(market_mock(0)));
        assert_ok!(MarketCommons::push_market(market_mock(1)));
        assert_ok!(MarketCommons::push_market(market_mock(2)));
        assert_eq!(MarketCommons::market(&0).unwrap().oracle, 0);
        assert_eq!(MarketCommons::market(&1).unwrap().oracle, 1);
        assert_eq!(MarketCommons::market(&2).unwrap().oracle, 2);
    });
}

#[test]
fn markets_storage_map_interacts_correctly_with_push_market() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(MarketCommons::push_market(market_mock(0)));
        assert_ok!(MarketCommons::push_market(market_mock(1)));
        assert_ok!(MarketCommons::push_market(market_mock(2)));
        assert_eq!(<Markets<Runtime>>::get(0).unwrap().oracle, 0);
        assert_eq!(<Markets<Runtime>>::get(1).unwrap().oracle, 1);
        assert_eq!(<Markets<Runtime>>::get(2).unwrap().oracle, 2);
    });
}

#[test]
fn market_fails_if_market_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(MarketCommons::market(&0), crate::Error::<Runtime>::MarketDoesNotExist);
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_noop!(MarketCommons::market(&3), crate::Error::<Runtime>::MarketDoesNotExist);
    });
}

#[test]
fn mutate_market_succeeds_if_closure_succeeds() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(MarketCommons::push_market(market_mock(0)));
        assert_ok!(MarketCommons::mutate_market(&0, |market| {
            market.oracle = 1;
            Ok(())
        }));
        assert_eq!(MarketCommons::market(&0).unwrap().oracle, 1);
    });
}

#[test]
fn mutate_market_fails_if_market_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            MarketCommons::mutate_market(&0, |_| Ok(())),
            crate::Error::<Runtime>::MarketDoesNotExist
        );
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_noop!(
            MarketCommons::mutate_market(&3, |_| Ok(())),
            crate::Error::<Runtime>::MarketDoesNotExist
        );
    });
}

#[test]
fn mutate_market_is_noop_if_closure_fails() {
    ExtBuilder::default().build().execute_with(|| {
        let err = DispatchError::Other("foo");
        assert_ok!(MarketCommons::push_market(market_mock(0)));
        assert_noop!(
            // We change the market to check that `mutate_market` is no-op when it errors.
            MarketCommons::mutate_market(&0, |market| {
                market.oracle = 1;
                Err(err)
            }),
            err
        );
    });
}

#[test]
fn remove_market_correctly_interacts_with_push_market() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(MarketCommons::push_market(market_mock(0)));
        assert_ok!(MarketCommons::push_market(market_mock(1)));
        assert_ok!(MarketCommons::push_market(market_mock(2)));

        assert_ok!(MarketCommons::remove_market(&1));
        assert_eq!(MarketCommons::market(&0).unwrap().oracle, 0);
        assert_noop!(MarketCommons::market(&1), crate::Error::<Runtime>::MarketDoesNotExist);
        assert_eq!(MarketCommons::market(&2).unwrap().oracle, 2);

        assert_ok!(MarketCommons::remove_market(&2));
        assert_eq!(MarketCommons::market(&0).unwrap().oracle, 0);
        assert_noop!(MarketCommons::market(&1), crate::Error::<Runtime>::MarketDoesNotExist);
        assert_noop!(MarketCommons::market(&2), crate::Error::<Runtime>::MarketDoesNotExist);

        assert_ok!(MarketCommons::remove_market(&0));
        assert_noop!(MarketCommons::market(&0), crate::Error::<Runtime>::MarketDoesNotExist);
        assert_noop!(MarketCommons::market(&1), crate::Error::<Runtime>::MarketDoesNotExist);
        assert_noop!(MarketCommons::market(&2), crate::Error::<Runtime>::MarketDoesNotExist);
    });
}

#[test]
fn remove_market_fails_if_market_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(MarketCommons::remove_market(&0), crate::Error::<Runtime>::MarketDoesNotExist);
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_noop!(MarketCommons::remove_market(&3), crate::Error::<Runtime>::MarketDoesNotExist);
    });
}

#[test]
fn insert_market_pool_fails_if_market_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            MarketCommons::insert_market_pool(0, 15),
            crate::Error::<Runtime>::MarketDoesNotExist
        );
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_noop!(
            MarketCommons::insert_market_pool(3, 12),
            crate::Error::<Runtime>::MarketDoesNotExist
        );
    });
}

#[test]
fn insert_market_pool_fails_if_market_has_a_pool() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_ok!(MarketCommons::insert_market_pool(0, 15));
        assert_noop!(
            MarketCommons::insert_market_pool(0, 14),
            crate::Error::<Runtime>::PoolAlreadyExists
        );
    });
}

#[test]
fn market_pool_correctly_interacts_with_insert_market_pool() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_ok!(MarketCommons::insert_market_pool(0, 15));
        assert_ok!(MarketCommons::insert_market_pool(1, 14));
        assert_ok!(MarketCommons::insert_market_pool(2, 13));
        assert_eq!(MarketCommons::market_pool(&0).unwrap(), 15);
        assert_eq!(MarketCommons::market_pool(&1).unwrap(), 14);
        assert_eq!(MarketCommons::market_pool(&2).unwrap(), 13);
    });
}

#[test]
fn market_pool_fails_if_market_has_no_pool() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            MarketCommons::market_pool(&0),
            crate::Error::<Runtime>::MarketPoolDoesNotExist
        );
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_ok!(MarketCommons::insert_market_pool(0, 15));
        assert_ok!(MarketCommons::insert_market_pool(1, 14));
        assert_ok!(MarketCommons::insert_market_pool(2, 13));
        assert_noop!(
            MarketCommons::market_pool(&3),
            crate::Error::<Runtime>::MarketPoolDoesNotExist
        );
    });
}

#[test]
fn remove_market_pool_correctly_interacts_with_insert_market_pool() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_ok!(MarketCommons::insert_market_pool(0, 15));
        assert_ok!(MarketCommons::insert_market_pool(1, 14));
        assert_ok!(MarketCommons::insert_market_pool(2, 13));

        assert_ok!(MarketCommons::remove_market_pool(&1));
        assert_eq!(MarketCommons::market_pool(&0).unwrap(), 15);
        assert_noop!(
            MarketCommons::market_pool(&1),
            crate::Error::<Runtime>::MarketPoolDoesNotExist
        );
        assert_eq!(MarketCommons::market_pool(&2).unwrap(), 13);

        assert_ok!(MarketCommons::remove_market_pool(&2));
        assert_eq!(MarketCommons::market_pool(&0).unwrap(), 15);
        assert_noop!(
            MarketCommons::market_pool(&1),
            crate::Error::<Runtime>::MarketPoolDoesNotExist
        );
        assert_noop!(
            MarketCommons::market_pool(&2),
            crate::Error::<Runtime>::MarketPoolDoesNotExist
        );

        assert_ok!(MarketCommons::remove_market_pool(&0));
        assert_noop!(
            MarketCommons::market_pool(&0),
            crate::Error::<Runtime>::MarketPoolDoesNotExist
        );
        assert_noop!(
            MarketCommons::market_pool(&1),
            crate::Error::<Runtime>::MarketPoolDoesNotExist
        );
        assert_noop!(
            MarketCommons::market_pool(&2),
            crate::Error::<Runtime>::MarketPoolDoesNotExist
        );
    });
}

#[test]
fn remove_market_pool_fails_if_market_has_no_pool() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            MarketCommons::remove_market_pool(&0),
            crate::Error::<Runtime>::MarketPoolDoesNotExist
        );
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_ok!(MarketCommons::insert_market_pool(0, 15));
        assert_ok!(MarketCommons::insert_market_pool(1, 14));
        assert_ok!(MarketCommons::insert_market_pool(2, 13));
        assert_noop!(
            MarketCommons::remove_market_pool(&3),
            crate::Error::<Runtime>::MarketPoolDoesNotExist
        );
    });
}

#[test]
fn market_counter_interacts_correctly_with_push_market_and_remove_market() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(<MarketCounter<Runtime>>::get(), 0);
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_eq!(<MarketCounter<Runtime>>::get(), 1);
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_eq!(<MarketCounter<Runtime>>::get(), 2);
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_eq!(<MarketCounter<Runtime>>::get(), 3);
        assert_ok!(MarketCommons::remove_market(&1));
        assert_eq!(<MarketCounter<Runtime>>::get(), 3);
        assert_ok!(MarketCommons::remove_market(&2));
        assert_eq!(<MarketCounter<Runtime>>::get(), 3);
        assert_ok!(MarketCommons::push_market(MARKET_DUMMY));
        assert_eq!(<MarketCounter<Runtime>>::get(), 4);
    });
}

fn market_mock(
    id: AccountIdTest,
) -> zeitgeist_primitives::types::Market<AccountIdTest, Balance, BlockNumber, Moment, Asset<MarketId>>
{
    let mut market = MARKET_DUMMY;
    market.oracle = id;
    market
}
