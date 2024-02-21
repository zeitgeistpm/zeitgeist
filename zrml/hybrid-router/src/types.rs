/// Represents the strategy used when placing an order in a trading environment.
pub enum Strategy {
    /// The trade is rolled back if it cannot be executed fully.
    ImmediateOrCancel,
    /// Partially fulfills the order if possible, placing the remainder in the order book. Favors
    /// achieving a specific price rather than immediate execution.
    LimitOrder,
}