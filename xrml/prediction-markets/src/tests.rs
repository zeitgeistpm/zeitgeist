use crate::{Error, mock::*};
use frame_support::{
    assert_noop, assert_ok,
    traits::{Currency, ReservableCurrency},
};
use sp_core::H256;
use xrml_traits::shares::{Shares as SharesTrait, ReservableShares};

#[test]
fn it_works() {
    ExtBuilder::default().build().execute_with(|| {
        // TODO
    });
}
