use crate::{mock::*, market::*, Error};
use frame_support::{
    assert_noop, assert_ok,
    dispatch::DispatchError,
};
use sp_core::H256;
use zrml_traits::shares::Shares as SharesTrait;

#[test]
fn it_creates_a_new_swap_internal() {
    ExtBuilder::default().build().execute_with(|| {
        
    });
}
