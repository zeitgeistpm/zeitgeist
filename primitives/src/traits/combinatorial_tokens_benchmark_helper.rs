use alloc::vec::Vec;

pub trait CombinatorialTokensBenchmarkHelper {
    type Balance;

    fn setup_payout_vector(payout: Option<Vec<Self::Balance>>);
}
