use crate::{
    mock::{runtime::Runtime, types::MockPayout},
    BalanceOf,
};
use alloc::vec::Vec;
use zeitgeist_primitives::traits::CombinatorialTokensBenchmarkHelper;

pub struct BenchmarkHelper;

impl CombinatorialTokensBenchmarkHelper for BenchmarkHelper {
    type Balance = BalanceOf<Runtime>;

    fn setup_payout_vector(payout: Option<Vec<Self::Balance>>) {
        MockPayout::set_return_value(payout);
    }
}
