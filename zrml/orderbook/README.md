# Orderbook Pallet

A pallet of an on-chain orderbook, which allows to exchange the market's base
asset for outcome assets and vice versa.

## Overview

The order book can be set as a market's scoring rule. 
It allows to place, partially or fully fill and remove orders.

## Terminology

- `maker_partial_fill`: The partial amount of what the maker wants to fill.
- `maker_fill`: The amount of what the maker wants to fill.
- `taker_fill`: The amount of what the taker wants to fill.
- `maker_asset`: The asset that the maker wants to sell.
- `maker_amount`: The amount of the asset that the maker wants to sell.
- `taker_asset`: The asset that the taker needs to have to buy the maker's asset.
- `taker_amount`: The amount of the asset that the taker needs to have to buy the
  maker's asset.

## Interface

### Dispatches

#### Public Dispatches

- `remove_order`: Allows a user to remove their order from the order book.
- `fill_order`: Used to fill an order either partially or completely.
- `place_order`: Places a new order into the order book.
