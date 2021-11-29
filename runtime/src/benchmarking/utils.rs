// This file was originally fetched from Acala
// https://github.com/AcalaNetwork/Acala/blob/6e0dae03040db2a1ef168a2ecba357c7b628874c/runtime/mandala/src/benchmarking/utils.rs

// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use crate::{AccountId, Balance, Currency, CurrencyId, Runtime};

use frame_support::assert_ok;
use orml_traits::MultiCurrencyExtended;
use sp_runtime::traits::{SaturatedConversion, StaticLookup};

pub fn lookup_of_account(
    who: AccountId,
) -> <<Runtime as frame_system::Config>::Lookup as StaticLookup>::Source {
    <Runtime as frame_system::Config>::Lookup::unlookup(who)
}

pub fn set_balance(currency_id: CurrencyId, who: &AccountId, balance: Balance) {
    assert_ok!(<Currency as MultiCurrencyExtended<_>>::update_balance(
        currency_id,
        who,
        balance.saturated_into()
    ));
}

#[cfg(test)]
pub mod tests {
    pub fn new_test_ext() -> sp_io::TestExternalities {
        frame_system::GenesisConfig::default().build_storage::<crate::Runtime>().unwrap().into()
    }
}
