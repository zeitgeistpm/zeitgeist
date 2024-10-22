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
