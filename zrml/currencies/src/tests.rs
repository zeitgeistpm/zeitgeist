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

#![cfg(test)]
use crate::mock::{Currencies, ExtBuilder, OrmlCurrencies};
use alloc::collections::BTreeSet;
use orml_traits::MultiCurrency;
use proptest::{collection::hash_set, prelude::*, strategy::ValueTree, test_runner::TestRunner};
use zeitgeist_primitives::{
    traits::ZeitgeistAssetManager,
    types::{AccountIdTest, Asset, CurrencyId, ScalarPosition},
};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]
    #[test]
    fn validate_storages( _i : u32) {
    ExtBuilder::default().build().execute_with(|| {
        let deposit_input = hash_set(1u128..5000u128, 100..101);
        let transfer_input = hash_set(5001u128..10000u128, 100..101);
        let mut runner = TestRunner::deterministic();
        let deposit_accounts = deposit_input.new_tree(&mut runner).unwrap().current();
        let transfer_accounts = transfer_input.new_tree(&mut runner).unwrap().current();
        let currency_id : CurrencyId = Asset::ScalarOutcome(0, ScalarPosition::Short);
        let balance =  100_u32;
        for (account_a, account_b) in deposit_accounts.iter().zip(transfer_accounts.iter()) {
            <Currencies as MultiCurrency<AccountIdTest>>::deposit(currency_id, account_a, balance.into()).expect("deposit failed");
            // transfer half balance to account_b
            <Currencies as MultiCurrency<AccountIdTest>>::transfer(currency_id, account_a, account_b, (balance/2).into()).expect("deposit failed");
            let account_from_currencies_zrml = <Currencies as ZeitgeistAssetManager<AccountIdTest>>::accounts_by_currency_id(currency_id).expect("accounts_by_currency_id failed");
            let account_from_currencies_orml = <OrmlCurrencies as ZeitgeistAssetManager<AccountIdTest>>::accounts_by_currency_id(currency_id).expect("accounts_by_currency_id failed");
            let unique_accounts_orml = BTreeSet::from_iter(account_from_currencies_orml.1.iter());
            let unique_accounts_zrml = BTreeSet::from_iter(account_from_currencies_zrml.1.iter());
            // any account in orml has balance for given currency,
            // must have been noted in zrml's storage
            for account in unique_accounts_orml {
                assert!(unique_accounts_zrml.contains(account));
            }
        }
    });
    }
}
