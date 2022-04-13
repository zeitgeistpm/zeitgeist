# v0.3.1

- Removed function parameter `keep_outcome_assets` from dispatchables
  `create_cpmm_market_and_deploy_assets` and
  `deploy_swap_pool_and_additional_liquidity` in prediction-markets pallet.
- Converted `base_asset` field of `Pool<Balance, MarketId>` from
  `Option<Asset<MarketId>>` to `Asset<MarketId>`. Pools with `base_asset` equal
  to `None` are migrated to `Asset::Ztg`.
