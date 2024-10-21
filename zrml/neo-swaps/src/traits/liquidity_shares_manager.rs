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

use crate::{BalanceOf, Config};
use sp_runtime::{DispatchError, DispatchResult};

/// Trait for managing pool share tokens and distributing fees to LPs according to their share of
/// the total issuance of pool share tokens.
pub trait LiquiditySharesManager<T: Config> {
    type JoinBenchmarkInfo;

    /// Add `amount` units of pool shares to the account of `who`.
    fn join(
        &mut self,
        who: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<Self::JoinBenchmarkInfo, DispatchError>;

    /// Remove `amount` units of pool shares from the account of `who`.
    fn exit(&mut self, who: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult;

    /// Transfer `amount` units of pool shares from `sender` to `receiver`.
    #[allow(unused)]
    fn split(
        &mut self,
        sender: &T::AccountId,
        receiver: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult;

    /// Deposit a lump sum of fees `amount` to the pool share holders.
    fn deposit_fees(&mut self, amount: BalanceOf<T>) -> DispatchResult;

    /// Withdraw and return the share of the fees belonging to `who`.
    fn withdraw_fees(&mut self, who: &T::AccountId) -> Result<BalanceOf<T>, DispatchError>;

    /// Return the pool shares balance of `who`.
    fn shares_of(&self, who: &T::AccountId) -> Result<BalanceOf<T>, DispatchError>;

    /// Return the total issuance of pool shares.
    fn total_shares(&self) -> Result<BalanceOf<T>, DispatchError>;
}
