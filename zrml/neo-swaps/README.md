# Neo Swaps Module

The Neo Swaps module implements liquidity pools which use the Logarithmic Market
Scoring Rule (LMSR) to determine spot prices and swap amounts, and allow users
to dynamically provide liquidity.

## Overview

For a detailed description of the underlying mathematics see [here][docslink].

### Terminology

- _Collateral_: The currency type that backs the outcomes in the pool. This is
  also called the _base asset_ in other contexts.
- _Exit_: Refers to removing (part of) the liquidity from a liquidity pool in
  exchange for burning pool shares.
- _External fees_: After taking swap fees, additional fees can be withdrawn from
  an informant's collateral. They might go to the chain's treasury or the market
  creator.
- _Join_: Refers to adding more liquidity to a pool and receiving pool shares in
  return.
- _Liquidity provider_: A user who owns pool shares indicating their stake in
  the liquidity pool.
- _Pool Shares_: A token indicating the owner's per rate share of the liquidity
  pool.
- _Reserve_: The balances in the liquidity pool used for trading.
- _Swap fees_: Part of the collateral paid or received by informants that is
  moved to a separate account owned by the liquidity providers. They need to be
  withdrawn using the `withdraw_fees` extrinsic.
- _Liquidity Tree_: A data structure used to store a pool's liquidity providers'
  positions.

### Liquidity Tree

The _liquidity tree_ is one implementation of the `LiquiditySharesManager` trait
which the `Pool` struct uses to manage liquidity provider's positions. Liquidity
shares managers in general handles how many pool shares\_ each LP owns (similar
to pallet-balances), as well as the distribution of fees.

The liquidity tree is a binary segment tree. Each node represents one liquidity
provider and stores their stake in the pool and how much fees they're owed. As
opposed to a naked list, the liquidity tree solves one particular problem:
Naively distributing fees every time a trade is executed requires
`O(liquidity_providers)` operations, which is unacceptable. The problem is
solved by lazily distributing fees using lazy propagation. Whenever changes are
made to a node in the tree, e.g. an LP joins, leaves or withdraws fees, fees are
then lazily propagated to the corresponding node of the tree before any other
changes are enacted. This brings the complexity of distributing fees to constant
time, while lazy propagation only requires `O(log(liquidity_providers))`
operations.

The design of the liquidity tree is based on
[Azuro-protocol/LiquidityTree](https://github.com/Azuro-protocol/LiquidityTree).

#### Lazy Propagation

Fees are propagated up the tree in a "lazy" manner, i.e., the propagation
happens when liquidity providers deposit liquidity, or withdraw liquidity or
fees. The process of lazy propagation at a node `node` is as follows:

```ignore
If node.descendant_stake == 0 then
    node.fees ← node.fees + node.lazy_fees
Else
    total_stake ← node.stake + node.descendant_stake
    fees ← (node.descendant_stake / total_stake) * node.lazy_fees
    node.fees ← node.fees + fees
    remaining ← node.lazy_fees - fees
    For each child in node.children() do
        child.lazy_fees ← child.lazy_fees + (child.descendant_stake / total_stake) * remaining
    End For
End If
node.lazy_fees ← 0
```

This means that at every node, the remaining lazy fees are distributed pro rata
between the current node and its two children. With the total stake defined as
the sum of the current node's stake and the stake of its descendants, the
process is as follows:

- The current node's fees are increased by `node.stake / total_stake` of the
  remaining lazy fees.
- Each child's lazy fees are increased by `child.descendant_stake / total_stake`
  of the remaining lazy fees.

### Notes

- The `Pool` struct tracks the reserve held in the pool account. The reserve
  changes when trades are executed or the liquidity changes, but the reserve
  does not take into account funds that are sent to the pool account
  unsolicitedly. This fixes a griefing vector which allows an attacker to
  manipulate prices by sending funds to the pool account.
- Pool shares are not recorded using the `ZeitgeistAssetManager` trait. Instead,
  they are part of the `Pool` object and can be tracked using events.
- When a pool is deployed, the pallet charges the signer an extra fee to the
  tune of the collateral's existential deposit. This fee is moved into the pool
  account (which holds the swap fees). This is done to ensure that small amounts
  of fees don't cause the entire transaction to fail with `ExistentialDeposit`.
  This "buffer" is burned when the pool is destroyed. The pool account is
  expected to be whitelisted from dusting for all other assets.

[docslink]: ./docs/docs.pdf
