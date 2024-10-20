pub trait FutarchyBenchmarkHelper<Oracle> {
    /// Creates an oracle which returns `value` when evaluated, provided that state is not modified
    /// any further.
    fn create_oracle(value: bool) -> Oracle;
}
