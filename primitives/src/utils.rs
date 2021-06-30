use frame_support::dispatch::{DispatchResultWithPostInfo, Weight};

pub fn calculate_actual_weight<F>(
    func: F,
    weight_parameter: u32,
    max_weight_parameter: u32,
) -> DispatchResultWithPostInfo
where
    F: Fn(u32) -> Weight,
{
    if weight_parameter == max_weight_parameter {
        Ok(None.into())
    } else {
        Ok(Some(func(weight_parameter)).into())
    }
}
