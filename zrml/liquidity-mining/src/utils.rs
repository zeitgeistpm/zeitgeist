use core::ops::Div;
use sp_runtime::traits::{CheckedDiv, Saturating, UniqueSaturatedInto};

// Per-thousand compared to `total_value` and a given `value`. For example, if total is 200,
// then 6 is 3% of 200.
//
// Results currently can't have more than 00.0% accuracy.
pub fn calculate_perthousand<T>(value: T, total_value: &T) -> Option<T>
where
    T: CheckedDiv + From<u16> + Saturating,
{
    let _1000_balance = T::from(1000u16);
    value.saturating_mul(_1000_balance).checked_div(total_value)
}

// The value compared to `total_value` and a given `perthousand`. For example, 3% of 200 is 6.
//
// Results currently can't have more than 00.0% accuracy.
pub fn calculate_perthousand_value<T>(perthousand: T, total_value: T) -> T
where
    T: Div<T, Output = T> + Saturating + From<u16>,
{
    let _1000_balance = T::from(1000u16);
    total_value.saturating_mul(perthousand) / _1000_balance
}

// Perthousand ranges from 0 to 1000 so it will never overflow.
pub fn perthousand_to_balance<B, T>(perthousand: T) -> B
where
    B: From<u16>,
    T: UniqueSaturatedInto<u16>,
{
    let ptd_u16: u16 = perthousand.unique_saturated_into();
    B::from(ptd_u16)
}
