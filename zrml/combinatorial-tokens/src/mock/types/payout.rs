// Copyright 2024-2025 Forecasting Technologies LTD.
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

use alloc::vec;
use core::cell::RefCell;
use zeitgeist_primitives::{
    traits::PayoutApi,
    types::{Balance, MarketId},
};

pub struct MockPayout;

impl MockPayout {
    pub fn set_return_value(value: Option<Vec<Balance>>) {
        PAYOUT_VECTOR_RETURN_VALUE.with(|v| *v.borrow_mut() = Some(value));
    }

    pub fn not_called() -> bool {
        PAYOUT_VECTOR_CALL_DATA.with(|values| values.borrow().is_empty())
    }

    pub fn called_once_with(expected: MarketId) -> bool {
        if PAYOUT_VECTOR_CALL_DATA.with(|values| values.borrow().len()) != 1 {
            return false;
        }

        let actual =
            PAYOUT_VECTOR_CALL_DATA.with(|value| *value.borrow().first().expect("can't be empty"));

        actual == expected
    }
}

impl PayoutApi for MockPayout {
    type Balance = Balance;
    type MarketId = MarketId;

    fn payout_vector(market_id: Self::MarketId) -> Option<Vec<Self::Balance>> {
        PAYOUT_VECTOR_CALL_DATA.with(|values| values.borrow_mut().push(market_id));

        PAYOUT_VECTOR_RETURN_VALUE
            .with(|value| value.borrow().clone())
            .expect("MockPayout: No return value configured")
    }
}

thread_local! {
    pub static PAYOUT_VECTOR_CALL_DATA: RefCell<Vec<MarketId>> = const { RefCell::new(vec![]) };
    pub static PAYOUT_VECTOR_RETURN_VALUE: RefCell<Option<Option<Vec<Balance>>>> = const { RefCell::new(None) };
}
