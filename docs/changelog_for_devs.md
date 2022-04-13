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

- The `PoolJoinSubsidy` and `PoolExitSubsidy` now have the following fields:
  `[asset, amount, pool_id, who]`. The amount refers to the amount of subsidy
  provided. When joining, the subsidy provider does _not_ immediately receive
  pool tokens, but the subsidy amount is reserved. When exiting, the subsidy
  amount is unreserved.

  When the pool holds enough subsidies at the end of the subsidy phase, the
  reserved funds are transferred to the pool, the pool shares are minted and the
  event `SubsidyCollected` is emitted, which now has the following fields:
  `[pool_id, vec![(who, pool_amount), ...], total_subsidy]`. The vector contains
  all subsidy providers and their amount of pool tokens.
