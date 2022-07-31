// This file was originally fetched from Acala
// https://github.com/AcalaNetwork/Acala/blob/6e0dae03040db2a1ef168a2ecba357c7b628874c/runtime/mandala/src/benchmarks/currencies.rs

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

use super::utils::{lookup_of_account, set_balance as update_balance};
use crate::{
    common::{AccountId, Balance, CurrencyId, Tokens},
    Runtime,
};
use frame_benchmarking::{account, whitelisted_caller};
use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;
use orml_traits::MultiCurrency;
use zeitgeist_primitives::{constants::BASE, types::Asset};

const SEED: u32 = 0;
const ASSET: CurrencyId = Asset::CategoricalOutcome(0, 0);

runtime_benchmarks! {
    { Runtime, orml_tokens }

    transfer {
        let amount: Balance = BASE;

        let from: AccountId = whitelisted_caller();
        update_balance(ASSET, &from, amount);

        let to: AccountId = account("to", 0, SEED);
        let to_lookup = lookup_of_account(to.clone());
    }: _(RawOrigin::Signed(from), to_lookup, ASSET, amount)
    verify {
        assert_eq!(<Tokens as MultiCurrency<_>>::total_balance(ASSET, &to), amount);
    }

    transfer_all {
        let amount: Balance = BASE;

        let from: AccountId = whitelisted_caller();
        update_balance(ASSET, &from, amount);

        let to: AccountId = account("to", 0, SEED);
        let to_lookup = lookup_of_account(to);
    }: _(RawOrigin::Signed(from.clone()), to_lookup, ASSET, false)
    verify {
        assert_eq!(<Tokens as MultiCurrency<_>>::total_balance(ASSET, &from), 0);
    }

    transfer_keep_alive {
        let from: AccountId = whitelisted_caller();
        update_balance(ASSET, &from, 2 * BASE);

        let to: AccountId = account("to", 0, SEED);
        let to_lookup = lookup_of_account(to.clone());
    }: _(RawOrigin::Signed(from), to_lookup, ASSET, BASE)
    verify {
        assert_eq!(<Tokens as MultiCurrency<_>>::total_balance(ASSET, &to), BASE);
    }

    force_transfer {
        let from: AccountId = account("from", 0, SEED);
        let from_lookup = lookup_of_account(from.clone());
        update_balance(ASSET, &from, 2 * BASE);

        let to: AccountId = account("to", 0, SEED);
        let to_lookup = lookup_of_account(to.clone());
    }: _(RawOrigin::Root, from_lookup, to_lookup, ASSET, BASE)
    verify {
        assert_eq!(<Tokens as MultiCurrency<_>>::total_balance(ASSET, &to), BASE);
    }

    set_balance {
        let who: AccountId = account("who", 0, SEED);
        let who_lookup = lookup_of_account(who.clone());

    }: _(RawOrigin::Root, who_lookup, ASSET, BASE, BASE)
    verify {
        assert_eq!(<Tokens as MultiCurrency<_>>::total_balance(ASSET, &who), 2 * BASE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmarks::utils::tests::new_test_ext;
    use orml_benchmarking::impl_benchmark_test_suite;

    impl_benchmark_test_suite!(new_test_ext(),);
}
