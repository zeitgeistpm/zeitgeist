use frame_support::pallet_prelude::Weight;

pub trait FutarchyOracle {
    /// Evaluates the query at the current block and returns the weight consumed and a `bool`
    /// indicating whether the query evaluated positively.
    fn evaluate(&self) -> (Weight, bool);
}
