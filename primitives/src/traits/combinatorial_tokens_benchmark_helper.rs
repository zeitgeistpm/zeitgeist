use alloc::vec::Vec;
use sp_runtime::DispatchResult;

pub trait CombinatorialTokensBenchmarkHelper {
    type Balance;

    fn setup_payout_vector(payout: Option<Vec<Self::Balance>>) -> DispatchResult;
}
