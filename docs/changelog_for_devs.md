# Changelog for Developers

Used for communicating changes to other parts of Zeitgeist infrastructure
([zeitgeistpm/ui](https://github.com/zeitgeistpm/ui),
[zeitgeistpm/sdk-next](https://github.com/zeitgeistpm/sdk-next),
[zeitgeistpm/zeitgeist-subsquid](https://github.com/zeitgeistpm/zeitgeist-subsquid))
and does not represent a complete changelog for the zeitgeistpm/zeitgeist
repository.

As of 0.3.9, the changelog's format is based on
<https://keepachangelog.com/en/1.0.0/> and ⚠️ marks changes that might break
components which query the chain's storage, the extrinsics or the runtime
APIs/RPC interface.

## v0.5.2

[#1310]: https://github.com/zeitgeistpm/zeitgeist/pull/1310
[#1307]: https://github.com/zeitgeistpm/zeitgeist/pull/1307

### Added

- ⚠️ [#1310] Add `market_id` field to `Market` struct.
- [#1310] Add `MarketBuilderTrait`, which is used to define
  `MarketCommonsPalletApi::build_market`, which should be used for creating
  markets in the future.
- [#1307] New hybrid router for managing the trade execution using the
  `neo-swaps` automated market maker and order book

  - `buy`: Routes a buy order to AMM and CDA to achieve the best average
    execution price.
  - `sell`: Routes a sell order to AMM and CDA to achieve the best average
    execution price.

  The new pallet has the following events:

  - `HybridRouterExecuted { tx_type, who, market_id, price_limit, asset_in, amount_in, asset_out, amount_out, external_fee_amount, swap_fee_amount }`:
    A trade was executed using the Hybrid Router.

  For details, please refer to the `README.md` and the in-file documentation.

### Deprectaed

- [#1310] `MarketCommonsPalletApi::push_market` is now deprecated.

## v0.5.1

[#1295]: https://github.com/zeitgeistpm/zeitgeist/pull/1295
[pallet-asset]:
  https://github.com/paritytech/polkadot-sdk/tree/master/substrate/frame/assets

### Added

- [#1295] New asset classes:
  - `CampaignAssetClass` - Can be registered by gov and council and be used in
    markets and to pay transaction fees.
  - `CustomAssetClass` - Allows any user to register their custom assets (can't
    be used in markets yet).
  - `MarketAssetClass` - Contains all asset types used in markets.
  - `CurrencyClass` - Contains all non-ztg currencies, currently only
    `ForeignAsset`. Temporarily also contains outcome asset types as they are
    being lazily migrated to `MarketAssetClass`
  - Subclasses (they are composites of multiple types from potentially various
    asset classes):
    - `BaseAssetClass` - Contains all assets that can be used as base asset /
      collateral in prediction markets.
    - `XcmAssetClass` - Contains all assets that can be transferred via XCM
      (used in XCM related pallets like XTokens).
    - `ParimutuelAssetClass` - Contains all assets that can be used in
      parimutuel markets.
  - All asset classes can be converted into the overarching asset class `Assets`
    (that contains all asset types) by using `Into` or simply decoding their
    scale codec representation into the `Assets` type.
  - `Assets` provides `TryInto` into all other asset classes, which fails if the
    asset type is not existent in the asset class.
- [#1295] Added [pallet-asset], which is a Substrate pallet that provides fine
  grained control over asset creation, destruction, management (mint, burn,
  freeze, admin account) and much more. It is used for `CampaignAssetClass`,
  `CustomAssetClass` and `MarketAssetClass`.
- [#1295] Added zrml-asset-router (AssetRouter). This pallet is an abstraction
  layer over multiple pallets (like orml-tokens and pallet-assets) that handles
  routing calls, managing asset destruction and the lazy migrating market assets
  from `CurrencyClass` to `MarketAssetClass`. It does not have any dispatchable
  calls or events, but custom errors that might be relayed to the dispatchable
  calls of the pallets that it uses within it's routing mechanism.
  `orml-currencies` (AssetManager) is ought to be used when interacting with the
  chain via transactions and can be used when developing pallets. In the latter
  case, some functionalities can only be used when directly interacting with
  zrml-asset-router.
- [#1295] Campaign assets have to be created and destroyed by gov or the
  council. Custom assets have to be created and destroyed via transactions.
  Market assets are automatically created and destroyed. In all non automatic
  cases, destroying is achieved by calling `start_destroy`.
- [#1295] Transaction fee payment is now possible with campaign assets. The fee
  is calculated as follows (with `CampaignAssetFeeMultiplier = 100`):

```rust
if ztg_supply / campaign_asset_supply >= 100 {
    return native_fee;
} else {
  return native_fee * campaign_asset_supply * 100 / ztg_supply;
}
```

### Changed

- [#1295] `Assets` does not contain the `CombinatorialOutcome` asset type
  anymore, but has been extended by all existing asset types.
- [#1295] The transaction fee asset type has been changed from `u32` to
  `Assets`.
- [#1295] The prediction market base asset type has been changed in the `Market`
  storage and market creation dispatchable calls to `BaseAssetClass`.
- [#1295] The asset type for XCM has been changed to `XcmAssetClass`. It is used
  in `orml-xtokens` (XTokens) and `orml-asset-registry` (AssetRegistry).

### Removed

- [#1295] `SerdeWrapper` has been removed.

### Deprecated

- [#1295] Market outcome asset types are no longer handled by `orml-tokens`
  (Tokens), except for existing markets which still used market asset types
  within `CurrencyClass`. `pallet-assets` (MarketAssets) now handles market
  outcome asset types from the `MarketAssetClass`.

## v0.5.0

[#1197]: https://github.com/zeitgeistpm/zeitgeist/pull/1197
[#1178]: https://github.com/zeitgeistpm/zeitgeist/pull/1178

### Changes

- ⚠️ Move the `zeitgeist_primitives::Pool` struct to `zrml_swaps::types::Pool` and change the following fields ([#1197]):
    - Remove `market_id`
    - Make `swap_fee` non-optional
    - Remove `total_subsidy`
    - Make `total_weight` non-optional
    - Make `weights` non-optional
- ⚠️ Change the type of `liquidity_shares_manager` in `zrml_neo_swaps::types::Pool` from `zrml_neo_swaps::types::SoloLp` to `zrml_neo_swaps::types::LiquidityTree`. Details on the liquidity tree can be found in the `README.md` of zrml-neo-swaps and the documentation of the `LiquidityTree` object ([#1179]).

### Migrations

- Closed all CPMM pools. Withdrawals are still allowed. Creating new pools will
  be impossible until further updates are deployed. ([#1197])
- Remove all Rikiddo storage elements. ([#1197])
- Migrate neo-swaps `Pools` storage. The market creator's liquidity position is translated into a position in the liquidity tree of the same value ([#1178]).

### Removed

- ⚠️ Remove the `Disputes` storage element from zrml-prediction-markets.
  ([#1197])
- ⚠️ Remove the following extrinsics from zrml-prediction-markets. ([#1197]):
  - `create_cpmm_market_and_deploy_assets`
  - `deploy_swap_pool_and_additional_liquidity`
  - `deploy_swap_pool_for_market`
- ⚠️ Remove the following config values from zrml-prediction-markets ([#1197]):
  - `MaxSubsidyPeriod`
  - `MinSubsidyPeriod`
  - `Swaps`
- Remove automatic arbitrage for CPMM pools. ([#1197])
- ⚠️ Remove the following extrinsics from zrml-swaps ([#1197]):
  - `admin_clean_up_pool`
  - `pool_exit_subsidy`
  - `pool_join_subsidy`
- ⚠️ Remove the following config values from zrml-swaps ([#1197]):
  - `FixedTypeU`
  - `FixedTypeS`
  - `LiquidityMining`
  - `MarketCommons`
  - `MinSubsidy`
  - `MinSubsidyPerAccount`
  - `RikiddoSigmoidFeeMarketEma`
- ⚠️ Remove `CPMM` and `RikiddoSigmoidFeeMarketEma` from `ScoringRule`.
  ([#1197])
- ⚠️ Remove `Suspended`, `CollectingSubsidy` and `InsufficientSubsidy` from
  `MarketStatus`. ([#1197])

### Deprecate

- ⚠️ Deprecate the following storage elements of zrml-prediction-markets (will
  be removed in v0.5.1; [#1197]):
  - `MarketIdsPerOpenBlock`
  - `MarketIdsPerOpenTimeFrame`
  - `MarketsCollectingSubsidy`
- ⚠️ Deprecate the following storage elements of zrml-market-commons (will be
  removed at an unspecified point in time; [#1197]):
  - `MarketPool`
- ⚠️ Deprecate the following storage elements of zrml-swaps (will be removed in
  v0.5.1; [#1197]):
  - `SubsidyProviders`
  - `PoolsCachedForArbitrage`

## v0.4.3

### Removed

- Remove old storage migrations

## v0.4.2

[#1127]: https://github.com/zeitgeistpm/zeitgeist/pull/1127
[#1148]: https://github.com/zeitgeistpm/zeitgeist/pull/1148
[#1138]: https://github.com/zeitgeistpm/zeitgeist/pull/1138

### Added

- Implement parimutuel market ([#1138]) maker to allow markets without liquidity
  provision. The new pallet has the following dispatchables:

  - `buy`: Buy outcome tokens.
  - `claim_rewards`: Claim the winner outcome tokens.
  - `claim_refunds`: Claim the refunds in case there was no winner.

  The new pallet has the following events:

  - `OutcomeBought { market_id, buyer, asset, amount_minus_fees, fees }`:
    Informant bought a position.
  - `RewardsClaimed { market_id, asset, balance, actual_payoff, sender }`:
    Informant claimed rewards.
  - `RefundsClaimed { market_id, asset, refunded_balance, sender }`: Informant
    claimed refunds.

  For details, please refer to the `README.md` and the in-file documentation.

### Added

- Add extrinsics to zrml-prediction-markets ([#1127]):
  - `schedule_early_close`: Schedule an early close of a market.
  - `dispute_early_close`: Dispute a scheduled early close of a market.
  - `reject_early_close`: Reject a scheduled early close of a market.
- Add events to zrml-prediction-markets ([#1127]):
  - `MarketEarlyCloseScheduled`: A market's early close was scheduled.
  - `MarketEarlyCloseDisputed`: A market's early close was disputed.
  - `MarketEarlyCloseRejected`: A market's early close was rejected.

### Changed

- Modify event `MarketDisputed` to have an additional field for the disputant.
  This is the account id of the user who called `dispute`. ([#1148])

## v0.4.1

### Added

- Implement AMM-2.0-light in the form of zrml-neo-swaps. The new pallet has the
  following dispatchables:

  - `buy`: Buy outcome tokens from the specified market.
  - `sell`: Sell outcome tokens to the specified market.
  - `join`: Join the liquidity pool for the specified market.
  - `exit`: Exit the liquidity pool for the specified market.
  - `withdraw_fees`: Withdraw swap fees from the specified market.
  - `deploy_pool`: Deploy a pool for the specified market and provide liquidity.

  The new pallet has the following events:

  - `BuyExecuted { who, market_id, asset_out, amount_in, amount_out, swap_fee_amount, external_fee_amount }`:
    Informant bought a position.
  - `SellExecuted { who, market_id, asset_in, amount_in, amount_out, swap_fee_amount, external_fee_amount }`:
    Informants sold a position.
  - `FeesWithdrawn { who }`: Liquidity provider withdrew fees.
  - `JoinExecuted { who, market_id, pool_shares_amount, amounts_in, new_liquidity_parameter }`:
    Liquidity provider joined the pool.
  - `ExitExecuted { who, market_id, pool_shares_amount, amounts_out, new_liquidity_parameter }`:
    Liquidity provider left the pool.
  - `PoolDeployed { who, market_id, pool_shares_amount, amounts_in, liquidity_parameter }`:
    Pool was created.
  - `PoolDestroyed { who, market_id, pool_shares_amount, amounts_out }`: Pool
    was destroyed.

  For details, please refer to the `README.md` and the in-file documentation.

## v0.4.0

[#976]: https://github.com/zeitgeistpm/zeitgeist/pull/976

### Changed

All things about Global Disputes Fix ⚠️ :

- Replace `WinnerInfo` by `GlobalDisputeInfo` with the following fields:
  - `winner_outcome: OutcomeReport`
  - `outcome_info: OutcomeInfo`
  - `status: GdStatus`

### Removed

All things about Global Disputes Fix ⚠️ :

- Remove the following event:
  - `OutcomeOwnersRewardedWithNoFunds`

### Added

- Add market creator incentives.
  - The following dispatchable calls within the prediction markets pallet now
    expect a market creator fee denoted as type `Perbill` after the `base_asset`
    parameter. The fee is bounded by the pallet's `Config` parameter
    `MaxCreatorFee`:
    - `create_market`
    - `create_cpmm_market_and_deploy_assets`
  - The market type now holds an additional field `creator_fee` using the type
    `Perbill` after the `creation` field.
  - The swaps pallet's `Config` parameter `MaxSwapFee` now is a boundary for the
    sum of all fees, currently the liqudity provider fee and the market creator
    fee. It is checked during the execution of the public function
    `create_pool`.
  - Fees are always transferred from the trader's account to the market
    creator's account either before or after the trade. The base asset is always
    preferred to pay fees. If the trade does not include the base asset, the
    pallet will try to convert the outcome asset to the base asset by executing
    a swap.
  - A new event `MarketCreatorFeesPaid` is emitted by the swaps pallet after
    successful payment of fees to the market creator. It contains the fields
    `\[payer, payee, amount, asset\]`.
- ⚠️ Add court production implementation ([#976]). Dispatchable calls are:
  - `join_court` - Join the court with a stake to become a juror in order to get
    the stake-weighted chance to be selected for decision making.
  - `delegate` - Join the court with a stake to become a delegator in order to
    delegate the voting power to actively participating jurors.
  - `prepare_exit_court` - Prepare as a court participant to leave the court
    system.
  - `exit_court` - Exit the court system in order to get the stake back.
  - `vote` - An actively participating juror votes secretely on a specific court
    case, in which the juror got selected.
  - `denounce_vote` - Denounce a selected and active juror, if the secret and
    vote is known before the actual reveal period.
  - `reveal_vote` - An actively participating juror reveals the previously
    casted secret vote.
  - `appeal` - After the reveal phase (aggregation period), the jurors decision
    can be appealed.
  - `reassign_juror_stakes` - After the appeal period is over, losers pay the
    winners for the jurors and delegators.
  - `set_inflation` - Set the yearly inflation rate of the court system. Events
    are:
  - `JurorJoined` - A juror joined the court system.
  - `ExitPrepared` - A court participant prepared to exit the court system.
  - `ExitedCourt` - A court participant exited the court system.
  - `JurorVoted` - A juror voted on a specific court case.
  - `JurorRevealedVote` - A juror revealed the previously casted secret vote.
  - `DenouncedJurorVote` - A juror was denounced.
  - `DelegatorJoined` - A delegator joined the court system.
  - `CourtAppealed` - A court case was appealed.
  - `MintedInCourt` - A court participant was rewarded with newly minted tokens.
  - `StakesReassigned` - The juror and delegator stakes have been reassigned.
    The losing jurors have been slashed. The winning jurors have been rewarded
    by the losers. The losing jurors are those, who did not vote, or did not
    vote with the plurality, were denounced or did not reveal their vote.
  - `InflationSet` - The yearly inflation rate of the court system was set.

All things about Global Disputes Fix ⚠️ :

- Add new dispatchable function:
  - `refund_vote_fees` - Return all vote funds and fees, when a global dispute
    was destroyed.
- Add the following events:
  - `OutcomeOwnerRewarded` for `Possession::Paid`
  - `OutcomeOwnersRewarded` for `Possession::Shared`
  - `OutcomesFullyCleaned` and `OutcomesPartiallyCleaned` for extrinsic
    `refund_vote_fees`
- Add enum `Possession` with variants:
- `Paid { owner: AccountId, fee: Balance }`
- `Shared { owners: BoundedVec }`
- `OutcomeInfo` has the following fields:
  - `outcome_sum: Balance`
  - `possession: Possession`
- Add `GdStatus` with the following enum variants:
  - `Active { add_outcome_end: BlockNumber, vote_end: BlockNumber }`
  - `Finished`
  - `Destroyed`

## v0.3.11

[#1049]: https://github.com/zeitgeistpm/zeitgeist/pull/1049

### Changed

- ⚠️ All tokens now use 10 fractional decimal places ([#1049]).
- Cross-consensus messages (XCM) assume the global canonical representation for
  token balances.
- The token metadata in the asset registry now assumes that the existential
  deposit and fee factor are stored in base 10,000,000,000.

## v0.3.10

[#1022]: https://github.com/zeitgeistpm/zeitgeist/pull/1022

### Added

- Use pallet-asset-tx-payment for allowing to pay transaction fees in foreign
  currencies ([#1022]). This requires each transaction to specify the fee
  payment token with `asset_id` (`None` is ZTG).

## v0.3.9

[#1011]: https://github.com/zeitgeistpm/zeitgeist/pull/1011
[#937]: https://github.com/zeitgeistpm/zeitgeist/pull/937
[#903]: https://github.com/zeitgeistpm/zeitgeist/pull/903

### Changed

- ⚠️ Add `outsider` field to `MarketBonds` struct. In particular, the `Market`
  struct's layout has changed ([#903]).
- Adjust `deposit` function used to calculate storage fees for the following
  pallets: identity, multisig, preimage, proxy. The cost of adding an identity
  reduced from a minimum of 125 ZTG to a minimum of 1.5243 ZTG ([#1011])

### Fixed

- ⚠️ Fix order of arguments for `get_spot_price` ([#937]).

## v0.3.8

- Added the `bonds` field to the `Market` struct, which tracks the status of the
  advisory, oracle and validity bonds. Each of its members has type `Bond`,
  which has three fields: `who` (the account that reserved the bond), `value`
  (the amount reserved), `is_settled` (a flag which determines if the bond was
  already unreserved and/or (partially) slashed).
- The market dispute mechanisms are now able to control their resolution. The
  `CorrectionPeriod` parameter determines how long the authorized pallet can
  call `authorize_market_outcome` again after the first call to it (fat-finger
  protection). The authority report now includes its resolution block number.
  This is the time of the first call to `authorize_market_outcome` plus the
  `CorrectionPeriod`.
- Create prediction markets with Ztg or registered foreign asset which has
  `allow_as_base_asset` set to `true` in `AssetRegistry` metadata. Extrinsics
  related to prediction market creation/editing now have `base_asset` parameter.

# v0.3.7

- Added on-chain arbitrage. See
  [ZIP-1](https://hackmd.io/@1ypDLjlbQ_e2Gp_1EW7kkg/BksyTQc-o) for details. When
  a pool is arbitraged, we emit one of the following events:
  `ArbitrageMintSell(pool_id, amount)`, `ArbitrageBuyBurn(pool_id, amount)` or
  `ArbitrageSkipped(pool_id)`. The latter can be safely ignored by the indexer.
  The `amount` parameter signifies the amount of funds moved into or out of the
  prize pool (mint-sell/buy-burn resp.) and the amount of full sets
  minted/burned. Note that in addition to these events, the low-level
  `tokens.Deposited` and `tokens.Transfer` events are also emitted.

- Added new pallet: GlobalDisputes. Dispatchable calls are:

  - `add_vote_outcome` - Add voting outcome to a global dispute in exchange for
    a constant fee. Errors if the voting outcome already exists or if the global
    dispute has not started or has already finished.
  - `vote_on_outcome` - Vote on existing voting outcomes by locking native
    tokens. Fails if the global dispute has not started or has already finished.
  - `unlock_vote_balance` - Return all locked native tokens in a global dispute.
    If the global dispute is not concluded yet the lock remains.
  - `purge_outcomes` - Purge all outcomes to allow the winning outcome owner(s)
    to get their reward. Fails if the global dispute is not concluded yet.
  - `reward_outcome_owner` - Reward the collected fees to the owner(s) of a
    voting outcome. Fails if not all outcomes are already purged. Events are:
  - `AddedVotingOutcome` (user added a voting outcome)
  - `GlobalDisputeWinnerDetermined` (finish the global dispute, set the winner)
  - `NonReward` (show that no reward exits)
  - `OutcomeOwnersRewarded` (reward process was finished and spent to outcome
    owners)
  - `OutcomesPartiallyCleaned` (outcomes storage item partially cleaned)
  - `OutcomesFullyCleaned` (outcomes storage item fully cleaned)
  - `VotedOnOutcome` (user voted on outcome)

- Authorized pallet now has `AuthorizedDisputeResolutionOrigin` hence
  `MarketDisputeMechanism::Authorized` does not need account_id. To create
  market with Authorized MDM specifying account_id for Authorized MDM is not
  required, any user satisfying `AuthorizedDisputeResolutionOrigin` can use
  Authorized MDM for resolving market.

- Properly configured reserve asset transfers via XCM:

  - Added xTokens pallet to transfer tokens accross chains
  - Added AssetRegistry pallet to register foreign asset
  - Added UnknownTokens pallet to handle unknown foreign assets
  - More information at <https://github.com/zeitgeistpm/zeitgeist/pull/661#top>

- Transformed integer scalar markets to fixed point with ten digits after the
  decimal point. As soon as this update is deployed, the interpretation of the
  scalar values must be changed.

- `reject_market` extrinsic now requires `reject_reason` parameter which is
  `Vec<u8>`. The config constant `MaxRejectReasonLen` defines maximum length of
  above parameter. `MarketRejected` event also contains `reject_reason` so that
  it can be cached for market creator.

- `request_edit` extrinsic added, which enables a user satisfying
  `RequestEditOrigin` to request edit in market with `Proposed` state, when
  successful it emits `MarketRequestedEdit` event. `request_edit` requires
  `edit_reason` parameter which is `Vec<u8>`. The config constant
  `MaxEditReasonLen` defines maximum length of above parameter. The
  `MarketRequestedEdit` event also contains `edit_reason`.

- `edit_market` extrinsic added, which enables creator of the market to edit
  market. It has same parameters as `create_market` except market_creation, on
  success it returns `MarketEdited` event.
- `get_spot_price()` RPC from Swaps support `with_fees` boolean parameter to
  include/exclude swap_fees in spot price, Currently this flag only works for
  CPMM.

# v0.3.6

- Added new field `deadlines` in Market structure, which has `grace_period`,
  `oracle_duration` and `dispute_duration` fields, all of which represent
  durations in number of blocks. The `create_market` extrinsic has a new
  parameter to specify these deadlines.
- Added `pallet-bounties` to the Zeitgeist runtime to facilitate community
  projects.
- Changed `MaxCategories` to `64`, as originally intended
- Changed the `reject_market` slash percentage of the `AdvisoryBond` from 100%
  to 0%; this value can be quickly adjusted in the future by using the new
  on-chain variable `AdvisoryBondSlashPercentage`.
- Temporarily disabled removal of losing assets when a market resolves.

# v0.3.5

- Added `Initialized` status for pools. A pool now starts in `Initialized`
  status and must be opened using `Swaps::open_pool`. While the pool is
  `Initialized`, it is allowed to call `pool_join` and `pool_exit`, but trading
  and single-asset operations are prohibited.
- Every asset in a pool has a minimum balance now that is:
  `max(0.01, ExistentialDeposit(Asset))`. Regarding the current configuration,
  every asset in the pool has a minimum balance of 0.01.
- A single member of the `AdvisoryCommittee` can now approve markets, whereas
  50% of all members have to agree upon rejecting a market.
- Implemented swap fees for CPMM pools. This means that the following extrinsics
  now have a (non-optional) `swap_fee` parameter:

  - `create_cpmm_market_and_deploy_assets`
  - `deploy_swap_pool_and_additional_liquidity`
  - `deploy_swap_pool_for_market`

  Furthermore, there's a maximum swap fee, specified by the `swaps` pallet's
  on-chain constant `MaxSwapFee`.

- Added new pallet: Styx. Dispatchable calls are:
  - `cross` - Burns native chain tokens to cross. In the case of Zeitgeist, this
    is granting the ability to claim your zeitgeist avatar.
  - `set_burn_amount(amount)` - Sets the new burn price for the cross. Intended
    to be called by governance.

# v0.3.4

- Changed the `weights` parameter of `deploy_swap_pool_and_additional_liquidity`
  and `deploy_swap_pool_for_market` to be a vector whose length is equal to the
  number of outcome tokens (one item shorter than before). The `weights` now
  refer to the weights of the outcome tokens only, and the user can no longer
  control the weight of the base asset. Instead, the weight of the base asset is
  equal to the sum of the weights of the outcome tokens, thereby ensuring that
  (assuming equal balances) the spot prices of the outcome tokens sum up to 1.
- Changed `Market` struct's `mdm` to `dispute_mechanism`.

# v0.3.3

- Introduced `MarketStatus::Closed`. Markets are automatically transitioned into
  this state when the market ends, and the `Event::MarketClosed` is emitted.
  Trading is not allowed on markets that are closed.

- Introduced `PoolStatus::Closed`; the pool of a market is closed when the
  market is closed. The `Event::PoolClosed` is emitted when this happens.

- Replace `PoolStatus::Stale` with `PoolStatus::Clean`. This state signals that
  the corresponding market was resolved and the losing assets deleted from the
  pool. The `Event::PoolCleanedUp` is emitted when the pool transitions into
  this state.

- Simplify `create_cpmm_market_and_deploy_assets`,
  `deploy_swap_pool_and_additional_liquidity` and `deploy_swap_pool_for_market`
  by using a single `amount` parameter instead of `amount_base_asset` and
  `amount_outcome_assets`.

- The `MarketCounter` of the `market-commons` pallet is incremented by one. This
  means that `MarketCounter` is now equal to the total number of markets ever
  created, instead of equal to the id of the last market created. For details
  regarding this fix, see <https://github.com/zeitgeistpm/zeitgeist/pull/636>
  and <https://github.com/zeitgeistpm/zeitgeist/issues/365>.

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
