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

use crate::{traits::DistributeFees, AssetOf, BalanceOf, Config, MarketIdOf};
use core::marker::PhantomData;
use orml_traits::MultiCurrency;
use sp_runtime::{DispatchError, SaturatedConversion};
use zrml_market_commons::MarketCommonsPalletApi;

pub struct MarketCreatorFee<T>(PhantomData<T>);

/// Uses the `creator_fee` field defined by the specified market to deduct a fee for the market's
/// creator. Calling `distribute` is noop if the market doesn't exist or the transfer fails for any
/// reason.
impl<T: Config> DistributeFees for MarketCreatorFee<T> {
    type Asset = AssetOf<T>;
    type AccountId = T::AccountId;
    type Balance = BalanceOf<T>;
    type MarketId = MarketIdOf<T>;

    fn distribute(
        market_id: Self::MarketId,
        asset: Self::Asset,
        account: Self::AccountId,
        amount: Self::Balance,
    ) -> Self::Balance {
        Self::impl_distribute(market_id, asset, account, amount)
            .unwrap_or_else(|_| 0u8.saturated_into())
    }
}

impl<T: Config> MarketCreatorFee<T> {
    fn impl_distribute(
        market_id: MarketIdOf<T>,
        asset: AssetOf<T>,
        account: T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let market = T::MarketCommons::market(&market_id)?; // Should never fail
        let fee_amount = market.creator_fee.mul_floor(amount);
        // Might fail if the transaction is too small
        T::MultiCurrency::transfer(asset, &account, &market.creator, fee_amount)?;
        Ok(fee_amount)
    }
}
