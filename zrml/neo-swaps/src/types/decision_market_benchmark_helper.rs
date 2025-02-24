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

use crate::{
    liquidity_tree::types::LiquidityTree,
    types::{DecisionMarketOracle, DecisionMarketOracleScoreboard, Pool, PoolType},
    BalanceOf, Config, MarketIdOf, Pallet, Pools,
};
use alloc::{collections::BTreeMap, vec};
use core::marker::PhantomData;
use sp_runtime::{traits::Zero, Saturating};
use zeitgeist_primitives::{
    math::fixed::{BaseProvider, ZeitgeistBase},
    traits::FutarchyBenchmarkHelper,
    types::Asset,
};

pub struct DecisionMarketBenchmarkHelper<T>(PhantomData<T>);

impl<T> FutarchyBenchmarkHelper<DecisionMarketOracle<T>> for DecisionMarketBenchmarkHelper<T>
where
    T: Config,
{
    /// Creates a mocked up pool with prices so that the returned decision market oracle evaluates
    /// to `value`. The pool is technically in invalid state.
    fn create_oracle(value: bool) -> DecisionMarketOracle<T> {
        let pool_id: MarketIdOf<T> = 0u8.into();
        let collateral = Asset::Ztg;

        // Create a `reserves` map so that `positive_outcome` has a higher price if and only if
        // `value` is `true`.
        let positive_outcome = Asset::CombinatorialToken([0u8; 32]);
        let negative_outcome = Asset::CombinatorialToken([1u8; 32]);
        let mut reserves = BTreeMap::new();
        let one: BalanceOf<T> = ZeitgeistBase::get().unwrap();
        let two: BalanceOf<T> = one.saturating_mul(2u8.into());
        if value {
            reserves.insert(positive_outcome, one);
            reserves.insert(negative_outcome, two);
        } else {
            reserves.insert(positive_outcome, two);
            reserves.insert(negative_outcome, one);
        }

        let scoreboard = DecisionMarketOracleScoreboard::new(
            Zero::zero(),
            Zero::zero(),
            Zero::zero(),
            Zero::zero(),
        );

        let account_id: T::AccountId = Pallet::<T>::pool_account_id(&pool_id);
        let pool = Pool {
            account_id: account_id.clone(),
            assets: vec![positive_outcome, negative_outcome].try_into().unwrap(),
            reserves: reserves.try_into().unwrap(),
            collateral,
            liquidity_parameter: one,
            liquidity_shares_manager: LiquidityTree::new(account_id, one).unwrap(),
            swap_fee: Zero::zero(),
            pool_type: PoolType::Standard(0u8.into()),
        };

        Pools::<T>::insert(pool_id, pool);

        DecisionMarketOracle::new(pool_id, positive_outcome, negative_outcome, scoreboard)
    }
}
