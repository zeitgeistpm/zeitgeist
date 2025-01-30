/// A trait for keeping track of a certain amount of work to be done.
pub trait CombinatorialTokensFuel {
    /// Creates a `Fuel` object from a `total` value which indicates the total amount of work to be
    /// done. This is usually done for benchmarking purposes.
    fn from_total(total: u32) -> Self;

    /// Returns a `u32` which indicates the total amount of work to be done. Must be `O(1)` to avoid
    /// excessive calculation if this call is used when calculating extrinsic weight.
    fn total(&self) -> u32;
}
