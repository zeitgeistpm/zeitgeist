#![cfg(test)]

use crate::{
    mock::{ExtBuilder, Runtime},
};
use frame_support::{
    //assert_err, 
    //assert_noop, 
    assert_ok,
    //dispatch::{DispatchError, DispatchResult},
};
// use test_case::test_case;

#[test]
fn create_runtime_ok() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        assert_ok!(Ok(()));
    });
}
