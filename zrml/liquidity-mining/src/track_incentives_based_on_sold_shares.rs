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

use crate::{
    utils::{calculate_perthousand, calculate_perthousand_value},
    BlockSoldShares, OwnedValues,
};
use core::marker::PhantomData;
use sp_runtime::traits::Saturating;

pub struct TrackIncentivesBasedOnSoldShares<T>(PhantomData<T>);

// No-one can have more balance than `Balance::MAX` so most functions saturate rewards.
impl<T> TrackIncentivesBasedOnSoldShares<T>
where
    T: crate::Config,
{
    pub(crate) fn exec() -> usize {
        let mut counter = 0;
        for (market_id, account_id, sold_shares) in <BlockSoldShares<T>>::iter() {
            let values = if let Ok(e) = <OwnedValues<T>>::try_get(market_id, account_id.clone()) {
                e
            } else {
                // Trying to retrieve an account that doesn't have any bought shares
                continue;
            };

            let ptd = if let Some(el) = calculate_perthousand(sold_shares, &values.total_shares) {
                el.into()
            } else {
                // `total_shares` is zero
                continue;
            };

            let balance_to_subtract = calculate_perthousand_value(ptd, values.total_incentives);

            <OwnedValues<T>>::mutate(market_id, account_id, |values| {
                // `total_balance` or `total_shares` can be less than 0 but this possible
                // scenario is ignored here.
                values.total_incentives =
                    values.total_incentives.saturating_sub(balance_to_subtract);
                values.total_shares = values.total_shares.saturating_sub(sold_shares);
            });
            counter = counter.saturating_add(1);
        }
        let _ = <BlockSoldShares<T>>::clear(u32::max_value(), None);
        counter
    }
}
