# Hybrid Router

## Overview

The Hybrid Router pallet provides a mechanism for routing limit orders to either
an automated market maker or an order book. The decision is made based on which
option would result in the most favourable execution price for the order.

### Terminology

- **Limit Order**: An order to buy or sell a certain quantity of an asset at a
  specified price or better.
- **Automated Market Maker (AMM)**: A type of decentralized exchange protocol
  that relies on a mathematical formula to price assets.
- **Order Book**: A list of buy and sell orders for a specific asset, organized
  by price level.
- **Strategy**: The strategy used when placing an order in a trading
  environment. Two strategies are supported: `ImmediateOrCancel` and
  `LimitOrder`.
- **TxType**: The type of transaction, either `Buy` or `Sell`.

### Features

- **Order Routing**: Routes orders to the most favourable execution venue.
- **Limit Order Support**: Supports the creation and execution of limit orders.
- **Integration**: Seamlessly integrates with both AMMs and order books.
- **Buy and Sell Orders**: Supports both buy and sell orders with a strategy to
  handle the remaining order when the price limit is reached.
- **Strategies**: Supports two strategies when placing an order:
  `ImmediateOrCancel` and `LimitOrder`.

### Usage

The Hybrid Router pallet provides two main functions: `buy` and `sell`. Both
functions take the following parameters:

- `market_id`: The ID of the market to buy from or sell on.
- `asset_count`: The number of assets traded on the market.
- `asset`: The asset to buy or sell.
- `amount_in`: The amount of the market's base asset to sell or the amount of
  `asset` to sell.
- `max_price` or `min_price`: The maximum price to buy at or the minimum price
  to sell at.
- `orders`: A list of orders from the book to use.
- `strategy`: The strategy to handle the remaining order when the `max_price` or
  `min_price` is reached.
