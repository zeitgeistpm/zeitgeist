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

use super::utils::{lookup_of_account, set_balance};
use crate::{
    AccountId, Amount, AssetManager, Balance, CurrencyId, ExistentialDeposit, GetNativeCurrencyId,
    Runtime,
};
use zeitgeist_primitives::{constants::BASE, types::Asset};

use frame_benchmarking::{account, whitelisted_caller};
use frame_system::RawOrigin;
use sp_runtime::traits::UniqueSaturatedInto;

use orml_benchmarking::runtime_benchmarks;
use orml_traits::MultiCurrency;

const SEED: u32 = 0;

const NATIVE: CurrencyId = GetNativeCurrencyId::get();
const ASSET: CurrencyId = Asset::CategoricalOutcome(0, 0);

runtime_benchmarks! {
    { Runtime, orml_currencies }

    // `transfer` non-native currency
    transfer_non_native_currency {
        let amount: Balance = 1_000 * BASE;
        let from: AccountId = whitelisted_caller();
        set_balance(ASSET, &from, amount);

        let to: AccountId = account("to", 0, SEED);
        let to_lookup = lookup_of_account(to.clone());
    }: transfer(RawOrigin::Signed(from), to_lookup, ASSET, amount)
    verify {
        assert_eq!(<AssetManager as MultiCurrency<_>>::total_balance(ASSET, &to), amount);
    }

    // `transfer` native currency and in worst case
    #[extra]
    transfer_native_currency_worst_case {
        let existential_deposit = ExistentialDeposit::get();
        let amount: Balance = existential_deposit.saturating_mul(1000);
        let from: AccountId = whitelisted_caller();
        set_balance(NATIVE, &from, amount);

        let to: AccountId = account("to", 0, SEED);
        let to_lookup = lookup_of_account(to.clone());
    }: transfer(RawOrigin::Signed(from), to_lookup, NATIVE, amount)
    verify {
        assert_eq!(<AssetManager as MultiCurrency<_>>::total_balance(NATIVE, &to), amount);
    }

    // `transfer_native_currency` in worst case
    // * will create the `to` account.
    // * will kill the `from` account.
    transfer_native_currency {
        let existential_deposit = ExistentialDeposit::get();
        let amount: Balance = existential_deposit.saturating_mul(1000);
        let from: AccountId = whitelisted_caller();
        set_balance(NATIVE, &from, amount);

        let to: AccountId = account("to", 0, SEED);
        let to_lookup = lookup_of_account(to.clone());
    }: _(RawOrigin::Signed(from), to_lookup, amount)
    verify {
        assert_eq!(<AssetManager as MultiCurrency<_>>::total_balance(NATIVE, &to), amount);
    }

    // `update_balance` for non-native currency
    update_balance_non_native_currency {
        let balance: Balance = 2 * BASE;
        let amount: Amount = balance.unique_saturated_into();
        let who: AccountId = account("who", 0, SEED);
        let who_lookup = lookup_of_account(who.clone());
    }: update_balance(RawOrigin::Root, who_lookup, ASSET, amount)
    verify {
        assert_eq!(<AssetManager as MultiCurrency<_>>::total_balance(ASSET, &who), balance);
    }

    // `update_balance` for native currency
    // * will create the `who` account.
    update_balance_native_currency_creating {
        let existential_deposit = ExistentialDeposit::get();
        let balance: Balance = existential_deposit.saturating_mul(1000);
        let amount: Amount = balance.unique_saturated_into();
        let who: AccountId = account("who", 0, SEED);
        let who_lookup = lookup_of_account(who.clone());
    }: update_balance(RawOrigin::Root, who_lookup, NATIVE, amount)
    verify {
        assert_eq!(<AssetManager as MultiCurrency<_>>::total_balance(NATIVE, &who), balance);
    }

    // `update_balance` for native currency
    // * will kill the `who` account.
    update_balance_native_currency_killing {
        let existential_deposit = ExistentialDeposit::get();
        let balance: Balance = existential_deposit.saturating_mul(1000);
        let amount: Amount = balance.unique_saturated_into();
        let who: AccountId = account("who", 0, SEED);
        let who_lookup = lookup_of_account(who.clone());
        set_balance(NATIVE, &who, balance);
    }: update_balance(RawOrigin::Root, who_lookup, NATIVE, -amount)
    verify {
        assert_eq!(<AssetManager as MultiCurrency<_>>::free_balance(NATIVE, &who), 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmarks::utils::tests::new_test_ext;
    use orml_benchmarking::impl_benchmark_test_suite;

    impl_benchmark_test_suite!(new_test_ext(),);
}
