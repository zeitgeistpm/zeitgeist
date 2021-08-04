use crate::mock::ExtBuilder;

#[test]
fn it_is_a_dummy_test() {
    ExtBuilder::default().build().execute_with(|| {
        assert!(true);
    });
}
