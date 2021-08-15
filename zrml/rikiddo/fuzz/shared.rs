use substrate_fixed::{FixedI128, FixedU128, types::extra::U33};

#[inline(always)]
pub(super) fn fixed_from_i128(from: i128) -> FixedI128<U33> {
    FixedI128::<U33>::from_ne_bytes(from.to_ne_bytes())
}

#[inline(always)]
pub(super) fn fixed_from_u128(from: u128) -> FixedU128<U33> {
    FixedU128::<U33>::from_ne_bytes(from.to_ne_bytes())
}