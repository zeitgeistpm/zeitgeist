use crate::pallet::{AssetOf, BalanceOf, Config};
use sp_runtime::DispatchError;

/// Trait for LMSR calculations and access to pool data.
pub(crate) trait PoolOperations<T: Config> {
    /// Return an ordered vector containing the assets held in the pool.
    fn assets(&self) -> Vec<AssetOf<T>>;

    /// Return `true` if the pool holds `asset`.
    fn contains(&self, asset: &AssetOf<T>) -> bool;

    /// Return the reserve of `asset` held in the pool.
    ///
    /// Beware! The reserve need not coincide with the balance in the pool account.
    fn reserve_of(&self, asset: &AssetOf<T>) -> Result<BalanceOf<T>, DispatchError>;

    /// Calculate the amount received from the swap that is executed when buying (the function
    /// `y(x)` from the documentation).
    ///
    /// Note that `y(x)` does not include the amount of `asset_out` received from buying complete
    /// sets and is therefore _not_ the total amount received from the buy.
    ///
    /// # Parameters
    ///
    /// - `asset_out`: The outcome being bought.
    /// - `amount_in`: The amount of collateral paid.
    fn calculate_swap_amount_out_for_buy(
        &self,
        asset_out: AssetOf<T>,
        amount_in: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError>;

    /// Calculate the amount receives from selling an outcome to the pool.
    ///
    /// # Parameters
    ///
    /// - `asset_in`: The outcome being sold.
    /// - `amount_in`: The amount of `asset_in` sold.
    fn calculate_swap_amount_out_for_sell(
        &self,
        asset_in: AssetOf<T>,
        amount_in: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError>;

    /// Calculate the spot price of `asset`.
    fn calculate_spot_price(&self, asset: AssetOf<T>) -> Result<BalanceOf<T>, DispatchError>;

    /// Calculate the maximum number of units of outcomes anyone is allowed to swap in or out of the
    /// pool.
    fn calculate_max_amount_in(&self) -> BalanceOf<T>;
}
