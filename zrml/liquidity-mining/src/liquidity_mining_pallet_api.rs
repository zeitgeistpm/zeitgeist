use frame_support::dispatch::DispatchResult;

/// Interface to interact with the Zeitgeist Liquidity Mining pallet.
pub trait LiquidityMiningPalletApi {
    type AccountId;
    type Balance;
    type BlockNumber;
    type MarketId;

    /// Increases the number of stored pool shares of an account for a given market.
    ///
    /// It is up to the caller to synchronize the amount of shares between different pallets
    fn add_shares(account_id: Self::AccountId, market_id: Self::MarketId, shares: Self::Balance);

    /// Removes a given `market_id` period from the storage distributing incentives to all
    /// related accounts.
    fn distribute_market_incentives(market_id: &Self::MarketId) -> DispatchResult;

    /// Decreases the number of stored pool shares of an account on a given market.
    ///
    /// It is up to the caller to synchronize the amount of shares between different pallets
    fn remove_shares(
        account_id: &Self::AccountId,
        market_id: &Self::MarketId,
        shares: Self::Balance,
    );
}
