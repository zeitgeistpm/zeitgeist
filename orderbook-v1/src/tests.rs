use crate::{Error, mock::*};
use frame_support::{
    assert_noop, assert_ok, traits::OnInitialize,
};

#[test]
fn it_works() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ok());
    });
}