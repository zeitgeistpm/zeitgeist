# Neo Swaps Module

The Neo Swaps module implements liquidity pools which use the Logarithmic Market
Scoring Rule (LMSR) to determine spot prices and swap amounts, and allow users
to dynamically provide liquidity.

## Overview

For a detailed description of the underlying mathematics see [here][docslink].

### Terminology

-   _Collateral_: The currency type that backs the outcomes in the pool. This is
    also called the _base asset_ in other contexts.
-   _Exit_: Refers to removing (part of) the liquidity from a liquidity pool in
    exchange for burning pool shares.
-   _External fees_: After taking swap fees, additional fees can be withdrawn
    from an informant's collateral. They might go to the chain's treasury or the
    market creator.
-   _Join_: Refers to adding more liquidity to a pool and receiving pool shares
    in return.
-   _Liquidity provider_: A user who owns pool shares indicating their stake in
    the liquidity pool.
-   _Pool Shares_: A token indicating the owner's per rate share of the
    liquidity pool.
-   _Reserve_: The balances in the liquidity pool used for trading.
-   _Swap fees_: Part of the collateral paid or received by informants that is
    moved to a separate account owned by the liquidity providers. They need to
    be withdrawn using the `withdraw_fees` extrinsic.

### Notes

-   The `Pool` struct tracks the reserve held in the pool account. The reserve
    changes when trades are executed or the liquidity changes, but the reserve
    does not take into account funds that are sent to the pool account
    unsolicitedly. This fixes a griefing vector which allows an attacker to
    change prices by sending funds to the pool account.
-   Pool shares are not recorded using the `ZeitgeistAssetManager` trait.
    Instead, they are part of the `Pool` object and can be tracked using events.
-   When the native currency is used as collateral, the pallet deposits the
    existential deposit to the pool account (which holds the swap fees). This is
    done to ensure that small amounts of fees don't cause the entire transaction
    to error with `ExistentialDeposit`. This "buffer" is removed when the pool
    is destroyed. The pool account is expected to be whitelisted from dusting
    for all other assets.

[docslink]: ./docs/docs.pdf
