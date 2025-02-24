// Copyright 2023-2025 Forecasting Technologies LTD.
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

use crate::{
    consts::EXP_NUMERICAL_LIMIT,
    math::{
        traits::{ComboMathOps, MathOps},
        types::{ComboMath, Math},
    },
    pallet::{AssetOf, BalanceOf, Config},
    traits::{LiquiditySharesManager, PoolOperations},
    types::PoolType,
    Error, MarketIdOf,
};
use alloc::{fmt::Debug, vec::Vec};
use frame_support::{
    storage::bounded_btree_map::BoundedBTreeMap, BoundedVec, CloneNoBound, PartialEqNoBound,
    RuntimeDebugNoBound,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{CheckedAdd, CheckedSub, Get},
    DispatchError, DispatchResult, SaturatedConversion, Saturating,
};
use zeitgeist_primitives::types::MarketStatus;
use zrml_market_commons::MarketCommonsPalletApi;

#[derive(
    CloneNoBound, Decode, Encode, Eq, MaxEncodedLen, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo,
)]
#[scale_info(skip_type_params(S, T))]
pub struct Pool<T, LSM, S>
where
    T: Config,
    LSM: Clone + Debug + LiquiditySharesManager<T> + PartialEq,
    S: Get<u32>,
{
    pub account_id: T::AccountId,
    pub assets: BoundedVec<AssetOf<T>, S>,
    pub reserves: BoundedBTreeMap<AssetOf<T>, BalanceOf<T>, S>,
    pub collateral: AssetOf<T>,
    pub liquidity_parameter: BalanceOf<T>,
    pub liquidity_shares_manager: LSM,
    pub swap_fee: BalanceOf<T>,
    pub pool_type: PoolType<MarketIdOf<T>, S>,
}

impl<T, LSM, S> PoolOperations<T> for Pool<T, LSM, S>
where
    T: Config,
    BalanceOf<T>: SaturatedConversion,
    LSM: Clone + Debug + LiquiditySharesManager<T> + TypeInfo + PartialEq,
    S: Get<u32>,
{
    fn assets(&self) -> Vec<AssetOf<T>> {
        self.assets.to_vec()
    }

    fn contains(&self, asset: &AssetOf<T>) -> bool {
        self.reserves.contains_key(asset)
    }

    fn reserve_of(&self, asset: &AssetOf<T>) -> Result<BalanceOf<T>, DispatchError> {
        Ok(*self.reserves.get(asset).ok_or(Error::<T>::AssetNotFound)?)
    }

    fn reserves_of(&self, assets: &[AssetOf<T>]) -> Result<Vec<BalanceOf<T>>, DispatchError> {
        assets.iter().map(|a| self.reserve_of(a)).collect()
    }

    /// Checks if the pool can be traded on.
    fn is_active(&self) -> Result<bool, DispatchError> {
        for market_id in self.pool_type.iter_market_ids() {
            let market = T::MarketCommons::market(market_id)?;

            if market.status != MarketStatus::Active {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn increase_reserve(
        &mut self,
        asset: &AssetOf<T>,
        increase_amount: &BalanceOf<T>,
    ) -> DispatchResult {
        let value = self.reserves.get_mut(asset).ok_or(Error::<T>::AssetNotFound)?;
        *value = value.checked_add(increase_amount).ok_or(Error::<T>::MathError)?;
        Ok(())
    }

    fn decrease_reserve(
        &mut self,
        asset: &AssetOf<T>,
        decrease_amount: &BalanceOf<T>,
    ) -> DispatchResult {
        let value = self.reserves.get_mut(asset).ok_or(Error::<T>::AssetNotFound)?;
        *value = value.checked_sub(decrease_amount).ok_or(Error::<T>::MathError)?;
        Ok(())
    }

    fn calculate_swap_amount_out_for_buy(
        &self,
        buy: Vec<AssetOf<T>>,
        sell: Vec<AssetOf<T>>,
        amount_in: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let reserves_buy = self.reserves_of(&buy)?;
        let reserves_sell = self.reserves_of(&sell)?;

        ComboMath::<T>::calculate_swap_amount_out_for_buy(
            reserves_buy,
            reserves_sell,
            amount_in,
            self.liquidity_parameter,
        )
    }

    fn calculate_swap_amount_out_for_sell(
        &self,
        buy: Vec<AssetOf<T>>,
        keep: Vec<AssetOf<T>>,
        sell: Vec<AssetOf<T>>,
        amount_buy: BalanceOf<T>,
        amount_sell: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let reserves_buy = self.reserves_of(&buy)?;
        let reserves_keep = self.reserves_of(&keep)?;
        let reserves_sell = self.reserves_of(&sell)?;

        ComboMath::<T>::calculate_swap_amount_out_for_sell(
            reserves_buy,
            reserves_keep,
            reserves_sell,
            amount_buy,
            amount_sell,
            self.liquidity_parameter,
        )
    }

    fn calculate_spot_price(&self, asset: AssetOf<T>) -> Result<BalanceOf<T>, DispatchError> {
        let reserve = self.reserve_of(&asset)?;
        Math::<T>::calculate_spot_price(reserve, self.liquidity_parameter)
    }

    fn calculate_numerical_threshold(&self) -> BalanceOf<T> {
        // Saturation is OK. If this saturates, the maximum amount in is just the numerical limit.
        self.liquidity_parameter.saturating_mul(EXP_NUMERICAL_LIMIT.saturated_into())
    }

    fn calculate_buy_ln_argument(
        &self,
        asset: AssetOf<T>,
        amount_in: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let reserve = self.reserve_of(&asset)?;
        Math::<T>::calculate_buy_ln_argument(reserve, amount_in, self.liquidity_parameter)
    }

    fn calculate_buy_amount_until(
        &self,
        asset: AssetOf<T>,
        until: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let reserve = self.reserve_of(&asset)?;
        let spot_price = Math::<T>::calculate_spot_price(reserve, self.liquidity_parameter)?;
        Math::<T>::calculate_buy_amount_until(until, self.liquidity_parameter, spot_price)
    }

    fn calculate_sell_amount_until(
        &self,
        asset: AssetOf<T>,
        until: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let reserve = self.reserve_of(&asset)?;
        let spot_price = Math::<T>::calculate_spot_price(reserve, self.liquidity_parameter)?;
        Math::<T>::calculate_sell_amount_until(until, self.liquidity_parameter, spot_price)
    }

    fn assets_complement(&self, assets: &[AssetOf<T>]) -> Vec<AssetOf<T>> {
        self.reserves.keys().filter(|a| !assets.contains(a)).cloned().collect()
    }
}
