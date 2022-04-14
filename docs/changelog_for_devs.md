# v0.3.1

- Removed function parameter `keep_outcome_assets` from dispatchables
  `create_cpmm_market_and_deploy_assets` and
  `deploy_swap_pool_and_additional_liquidity` in prediction-markets pallet.
- Converted `base_asset` field of `Pool<Balance, MarketId>` from
  `Option<Asset<MarketId>>` to `Asset<MarketId>`. Pools with `base_asset` equal
  to `None` are migrated to `Asset::Ztg`.
- Changed the following events to include a `pool_amount` field which specifies
  the amount of pool shares being minted or burned:

  - `PoolExit` (pool shares burned)
  - `PoolExitWithExactAssetAmount` (pool shares burned)
  - `PoolExitWithExactPoolAmount` (pool shares burned)
  - `PoolJoin` (pool shares minted)
  - `PoolJoinWithExactPoolAmount` (pool shares minted)
  - `PoolJoinWithExactAssetAmount` (pool shares minted)
  - `PoolCreate` (pool shares minted)

  The pool shares are always burned/minted for `who`.

- [Rikiddo-related] `PoolExitSubsidy` now has the following fields:
  `[asset, bound, pool_id, who, amount]`. The `bound` refers to the amount of
  subsidy the user is trying to withdraw, the `amount` is the amount actually
  withdrawn. When joining, the subsidy provider does _not_ immediately receive
  pool tokens, but the subsidy amount is reserved. When exiting, the subsidy
  amount is unreserved.

  When the pool holds enough subsidies at the end of the subsidy phase, the
  reserved funds are transferred to the pool, the pool shares are minted and the
  event `SubsidyCollected` is emitted, which now has the following fields:
  `[pool_id, vec![(who, pool_amount), ...], total_subsidy]`. The vector contains
  all subsidy providers and their amount of pool tokens.
