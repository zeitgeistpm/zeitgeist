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

use crate::pallet::*;

impl<T: Config> MultiCurrencyExtended<T::AccountId> for Pallet<T> {
    type Amount = <T::Currencies as MultiCurrencyExtended<T::AccountId>>::Amount;

    fn update_balance(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        by_amount: Self::Amount,
    ) -> DispatchResult {
        if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
            return <T::Currencies as MultiCurrencyExtended<T::AccountId>>::update_balance(
                currency, who, by_amount,
            );
        }

        if by_amount.is_zero() {
            return Ok(());
        }

        // Ensure that no overflows happen during abs().
        let by_amount_abs = if by_amount == Self::Amount::min_value() {
            return Err(Error::<T>::AmountIntoBalanceFailed.into());
        } else {
            by_amount.abs()
        };

        let by_balance = TryInto::<Self::Balance>::try_into(by_amount_abs)
            .map_err(|_| Error::<T>::AmountIntoBalanceFailed)?;
        if by_amount.is_positive() {
            Self::deposit(currency_id, who, by_balance)
        } else {
            Self::withdraw(currency_id, who, by_balance).map(|_| ())
        }
    }
}
