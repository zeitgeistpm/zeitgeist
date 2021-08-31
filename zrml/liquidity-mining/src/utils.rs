use crate::MomentOf;
use core::ops::{Div, Range};
use sp_runtime::{
    traits::{CheckedDiv, Saturating, UniqueSaturatedInto},
    SaturatedConversion,
};
use zeitgeist_primitives::constants::MILLISECS_PER_BLOCK;

// Calculates the **average** number of blocks occurred between the starting and ending time period
// of a market.
//
// To convert the block number type to the moment type, is is necessary to first convert the
// block number value to `u32`, which caps the maximum output to `u32::MAX`. Since this function
// is only used to evaluate perpetual balances, such limitation shouldn't be a problem.
pub fn calculate_average_blocks_of_a_time_period<T>(range: &Range<MomentOf<T>>) -> T::BlockNumber
where
    T: crate::Config,
{
    let total_value_time = range.end.saturating_sub(range.start);
    let mpb_balance = MILLISECS_PER_BLOCK.into();
    // The following won't overflow because `MILLISECS_PER_BLOCK` is not zero.
    let total_value_blocks = total_value_time / mpb_balance;
    let total_value_blocks_u32: u32 = total_value_blocks.saturated_into();
    total_value_blocks_u32.into()
}

// Per-thousand compared to `total_value` and a given `value`. For example, if total is 200,
// then 6 is 3% of 200.
//
// Results currently can't have more than 00.0% accuracy.
pub fn calculate_perthousand<T>(value: T, total_value: &T) -> Option<u16>
where
    T: CheckedDiv + From<u16> + Saturating + UniqueSaturatedInto<u16>,
{
    let _1000_balance = T::from(1000u16);
    let opaque = value.saturating_mul(_1000_balance).checked_div(total_value)?;
    Some(opaque.unique_saturated_into())
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
