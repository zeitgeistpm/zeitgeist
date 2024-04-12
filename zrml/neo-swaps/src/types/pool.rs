// Copyright 2023-2024 Forecasting Technologies LTD.
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
    math::{Math, MathOps},
    pallet::{AssetOf, BalanceOf, Config},
    traits::{LiquiditySharesManager, PoolOperations},
    Error,
};
use alloc::{fmt::Debug, vec::Vec};
use frame_support::{
    storage::bounded_btree_map::BoundedBTreeMap, CloneNoBound, PartialEqNoBound,
    RuntimeDebugNoBound,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{CheckedAdd, CheckedSub, Get},
    DispatchError, DispatchResult, SaturatedConversion, Saturating,
};

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
    pub reserves: BoundedBTreeMap<AssetOf<T>, BalanceOf<T>, S>,
    pub collateral: AssetOf<T>,
    pub liquidity_parameter: BalanceOf<T>,
    pub liquidity_shares_manager: LSM,
    pub swap_fee: BalanceOf<T>,
}

impl<T, LSM, S> PoolOperations<T> for Pool<T, LSM, S>
where
    T: Config,
    BalanceOf<T>: SaturatedConversion,
    LSM: Clone + Debug + LiquiditySharesManager<T> + TypeInfo + PartialEq,
    S: Get<u32>,
{
    fn assets(&self) -> Vec<AssetOf<T>> {
        self.reserves.keys().cloned().collect()
    }

    fn contains(&self, asset: &AssetOf<T>) -> bool {
        self.reserves.contains_key(asset)
    }

    fn reserve_of(&self, asset: &AssetOf<T>) -> Result<BalanceOf<T>, DispatchError> {
        Ok(*self.reserves.get(asset).ok_or(Error::<T>::AssetNotFound)?)
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
        asset_out: AssetOf<T>,
        amount_in: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let reserve = self.reserve_of(&asset_out)?;
        Math::<T>::calculate_swap_amount_out_for_buy(reserve, amount_in, self.liquidity_parameter)
    }

    fn calculate_swap_amount_out_for_sell(
        &self,
        asset_in: AssetOf<T>,
        amount_in: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let reserve = self.reserve_of(&asset_in)?;
        Math::<T>::calculate_swap_amount_out_for_sell(reserve, amount_in, self.liquidity_parameter)
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
}
