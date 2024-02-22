# Orderbook Pallet

A pallet of an on-chain order book, which allows to exchange the market's base
asset for outcome assets and vice versa.

## Overview

The order book can be set as a market's scoring rule. It allows to place,
partially or fully fill and remove orders.

## Terminology

- `maker_partial_fill`: The partial amount of what the maker wants to get filled.
- `maker_fill`: The amount of what the maker wants to get filled.
- `taker_fill`: The amount of what the taker wants to fill.
- `maker_asset`: The asset that the maker wants to sell.
- `maker_amount`: The amount of the asset that the maker wants to sell.
- `taker_asset`: The asset that the taker needs to have to buy the maker's
  asset.
- `taker_amount`: The amount of the asset that the taker needs to have to buy
  the maker's asset.

## Notes

- Orders must always bid or ask for the corresponding market's base asset.
- External fees are always paid in the market's base asset after the order is
  filled. In particular, the recipient of the collateral pays the fee. The
  implementation, however, arranges the transfers slightly differently for
  convenience:
  - If the order is an ask (maker sells outcome tokens), then the external fees
    are taken (not charged!) from the taker before the order is executed. The
    taker still receives the full amount of outcome tokens, but the maker
    receives only an adjusted amount.
  - If the order is a bid (maker buys outcome tokens), then the external fees
    are charged from the taker after the transaction is executed. In particular,
    the maker still receives the full amount of outcome tokens.

## Interface

### Dispatches

#### Public Dispatches

- `remove_order`: Allows a user to remove their order from the order book.
- `fill_order`: Used to fill an order either partially or completely.
- `place_order`: Places a new order into the order book.
