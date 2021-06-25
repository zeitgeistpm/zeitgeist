#![cfg(test)]

use crate::mock::*;

// TODO: Test fee calculation + different overflow scenarios + default values

#[test]
fn it_is_a_dummy_test() {
    ExtBuilder::default().build().execute_with(|| {
        assert!(true);
    });
}
