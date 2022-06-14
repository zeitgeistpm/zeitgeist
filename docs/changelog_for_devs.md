# v0.3.3

- Introduced `MarketStatus::Closed`. Markets are automatically transitioned into
  this state when the market ends, and the event `Event::MarketClosed` is
  emitted. Trading is not allowed on markets that are closed.

- Introduced `PoolStatus::Closed`; the pool of a market is closed when the
  market is closed. The event `PoolClosed` is emitted when this happens.

- Replace `PoolStatus::Stale` with `PoolStatus::Clean`. This state signals that
  the corresponding market was resolved and the losing assets deleted from the
  pool. The event `PoolCleanedUp` is emitted when the pool transitions into this
  state.

- Simplify `create_cpmm_market_and_deploy_assets`,
  `deploy_swap_pool_and_additional_liquidity` and `deploy_swap_pool_for_market`
  by using a single `amount` parameter instead of `amount_base_asset` and
  `amount_outcome_assets`.

- The `MarketCounter` of the `market-commons` pallet is incremented by one. This
  means that `MarketCounter` is now equal to the total number of markets ever
  created, instead of equal to the id of the last market created. For details
  regarding this fix, see https://github.com/zeitgeistpm/zeitgeist/pull/636 and
  https://github.com/zeitgeistpm/zeitgeist/issues/365.

- Made the `min_asset_amount_out` and `max_price` parameters of
  `swap_exact_amount_in` and the `max_asset_amount_in` and `max_price`
  parameters of `swap_exact_amount_out` optional

- Replaced `create_categorical_market` and `create_scalar_market` with
  `create_market`, which uses a `market_type` parameter to determine what market
  we create

# v0.3.2

- Added a field for `market_account` to `MarketCreated` event and `pool_account`
  field to `PoolCreate` event.

- Changed all call parameters of type `u16`, `BalanceOf`, `MarketId` and
  `PoolId` in extrinsics to
  [compact encoding](https://docs.substrate.io/v3/advanced/scale-codec/#compactgeneral-integers).

- Removed the `cancel_pending_market` function and the corresponding event
  `MarketCancelled`.

- Renamed `admin_set_pool_as_stale` to `admin_set_pool_to_stale` and changed the
  call parameters: Instead of specifying a `market_type` and `pool_id`, we now
  use a `market_id`. This fixes a problem where it's possible to specify an
  incorrect market type.

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

  If the pool holds enough subsidies at the end of the subsidy phase, the
  reserved funds are transferred to the pool, the pool shares are minted and the
  event `SubsidyCollected` is emitted, which now has the following fields:
  `[pool_id, vec![(who, pool_amount), ...], total_subsidy]`. The vector contains
  all subsidy providers and their amount of pool tokens.

  If, on the other hand, the pool does not hold sufficient subsidies, then
  `PoolDestroyedInSubsidyPhase` is emitted. This event has the following fields:
  `[pool_id, vec![(who, subsidy), ...]]`. The second element `subsidy` of each
  tuple refers to the amount of unreserved funds. No pool tokens are minted or
  burned.

- We now use a fork of ORML which emits the `Slashed` event when assets are
  slashed. This means that all balances changes are covered using only basic
  ORML events. We still emit custom events to inform about the semantic meaning
  of the ORML low-level events. The ORML events should be used to track
  balances.

- `create_market` will now reject markets that use
  `ScoringRule::RikiddoSigmoidFeeMarketEma` or `MarketDisputeMechanism::Court`.
  During the next release we plan to separate the Battery Station testnet and
  the mainnet runtimes. With this change we will reduce this restrictions to
  mainnet only.

- `LiquidityMining` was replaced with a dummy implementation and the calls to
  that pallet are filtered. We also plan to losen this constraint with the
  introduction of separate runtimes.

- When tokens are redeemed an event is emitted: `TokensRedeemed`. The fields are
  (in that order): `market_id, currency_id, amount_redeemed, payout, who`. This
  should also be regarded as an informative event, as stated before in this
  document balance changes should only be executed by events emitted by the
  pallets that manage the balances.
