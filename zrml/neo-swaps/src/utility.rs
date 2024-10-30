use sp_runtime::SaturatedConversion;

pub(crate) trait LogCeil {
    fn log_ceil(&self) -> Self;
}

impl LogCeil for u16 {
    fn log_ceil(&self) -> Self {
        let x = *self;

        let bits_minus_one = u16::MAX.saturating_sub(1);
        let leading_zeros: u16 = x.leading_zeros().saturated_into();
        let floor_log2 = bits_minus_one.saturating_sub(leading_zeros);

        if x.is_power_of_two() {
            floor_log2
        } else {
            floor_log2.saturating_add(1)
        }
    }
}
