#![cfg(all(feature = "mock", test))]

mod split_position;

use crate::mock::{
    ext_builder::ExtBuilder,
    runtime::{CombinatorialTokens, Currencies, Runtime, RuntimeOrigin},
};
use frame_support::assert_noop;
use orml_traits::MultiCurrency;
use sp_runtime::DispatchError;
use zeitgeist_primitives::{
    constants::base_multiples::*,
    types::{AccountIdTest, Asset, Balance, MarketId},
};

// For better readability of index sets.
pub(crate) const _0: bool = false;
pub(crate) const _1: bool = true;

pub(crate) struct Account {
    id: AccountIdTest,
}

impl Account {
    // TODO Not a pressing issue, but double booking accounts should be illegal.
    pub(crate) fn new(id: AccountIdTest) -> Account {
        Account { id }
    }

    /// Deposits `amount` of `asset` and returns the account to allow call chains.
    pub(crate) fn deposit(
        self,
        asset: Asset<MarketId>,
        amount: Balance,
    ) -> Result<Account, DispatchError> {
        Currencies::deposit(asset, &self.id, amount).map(|_| self)
    }

    pub(crate) fn signed(&self) -> RuntimeOrigin {
        RuntimeOrigin::signed(self.id)
    }

    pub(crate) fn free_balance(&self, asset: Asset<MarketId>) -> Balance {
        Currencies::free_balance(asset, &self.id)
    }
}
