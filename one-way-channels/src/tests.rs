use crate::{mock::*};
// use frame_support::{assert_ok, assert_noop};

#[test]
fn it_runs_a_test() {
	new_test_ext().execute_with(|| {
		assert_eq!(true, true);
	});
}
