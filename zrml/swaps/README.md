# Swaps Module

Legacy module which implements Balancer style liquidity pools.

## Overview

See the [Balancer whitepaper](https://www.balancer.fi/whitepaper.pdf) for
details.

### Terminology

-   _Exit_: Refers to removing (part of) the liquidity from a liquidity pool in
    exchange for burning pool shares. Amounts received by the LP are
    proportional to their ownership of the pool.
-   _Join_: Refers to adding more liquidity to a pool and receiving pool shares
    in return. Amounts moved into the pool are proportional to the LP's
    ownership of the pool.
-   _Liquidity provider_: A user who owns pool shares indicating their stake in
    the liquidity pool.
-   _Pool Shares_: A token indicating the owner's per rate share of the
    liquidity pool.
-   _Single Asset Operation_: An operation which combines joining or withdrawing
    from the pool and selling unwanted tokens. The end result is like entering
    (resp. leaving) the pool but paying (resp. receiving payment) in a single
    asset.
-   _Swap fees_: Part of the collateral paid or received by informants that is
    moved to a separate account owned by the liquidity providers. They need to
    be withdrawn using the `withdraw_fees` extrinsic.

### Exact Amounts

Almost all operations come in two variants, one where the assets entering the
pool are exact, the other where the assets leaving the pool are exact. Due to
the permissionless nature of the pallet, if multiple trades are placed in the
same block, the non-exact amount might slip for some of the parties.
