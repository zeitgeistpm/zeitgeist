// Copyright 2023 Forecasting Technologies LTD.
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

use crate::{traits::LiquiditySharesManager, BalanceOf, Config, Error};
use frame_support::ensure;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, CheckedAdd, CheckedSub, Zero},
    DispatchError, DispatchResult, RuntimeDebug,
};

// Deprecated as of v0.5.0. TODO Remove in 0.5.1!
#[derive(TypeInfo, MaxEncodedLen, Clone, Encode, Eq, Decode, PartialEq, RuntimeDebug)]
#[scale_info(skip_type_params(T))]
pub struct SoloLp<T: Config> {
    pub owner: T::AccountId,
    pub total_shares: BalanceOf<T>,
    pub fees: BalanceOf<T>,
}

#[allow(dead_code)]
impl<T: Config> SoloLp<T> {
    pub(crate) fn new(owner: T::AccountId, total_shares: BalanceOf<T>) -> SoloLp<T> {
        SoloLp { owner, total_shares, fees: Zero::zero() }
    }
}

impl<T: Config + frame_system::Config> LiquiditySharesManager<T> for SoloLp<T>
where
    T::AccountId: PartialEq<T::AccountId>,
    BalanceOf<T>: AtLeast32BitUnsigned + Copy + Zero,
{
    type JoinBenchmarkInfo = ();

    fn join(&mut self, who: &T::AccountId, shares: BalanceOf<T>) -> Result<(), DispatchError> {
        ensure!(*who == self.owner, Error::<T>::NotAllowed);
        self.total_shares = self.total_shares.checked_add(&shares).ok_or(Error::<T>::MathError)?;
        Ok(())
    }

    fn exit(&mut self, who: &T::AccountId, shares: BalanceOf<T>) -> DispatchResult {
        ensure!(*who == self.owner, Error::<T>::NotAllowed);
        ensure!(shares <= self.total_shares, Error::<T>::InsufficientPoolShares);
        self.total_shares = self.total_shares.checked_sub(&shares).ok_or(Error::<T>::MathError)?;
        Ok(())
    }

    fn split(
        &mut self,
        _sender: &T::AccountId,
        _receiver: &T::AccountId,
        _amount: BalanceOf<T>,
    ) -> DispatchResult {
        Err(Error::<T>::NotImplemented.into())
    }

    fn deposit_fees(&mut self, amount: BalanceOf<T>) -> DispatchResult {
        self.fees = self.fees.checked_add(&amount).ok_or(Error::<T>::MathError)?;
        Ok(())
    }

    fn withdraw_fees(&mut self, who: &T::AccountId) -> Result<BalanceOf<T>, DispatchError> {
        ensure!(*who == self.owner, Error::<T>::NotAllowed);
        let result = self.fees;
        self.fees = Zero::zero();
        Ok(result)
    }

    fn shares_of(&self, who: &T::AccountId) -> Result<BalanceOf<T>, DispatchError> {
        ensure!(*who == self.owner, Error::<T>::NotAllowed);
        Ok(self.total_shares)
    }

    fn total_shares(&self) -> Result<BalanceOf<T>, DispatchError> {
        Ok(self.total_shares)
    }
}
