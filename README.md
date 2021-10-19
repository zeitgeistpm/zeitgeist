<a href="https://zeitgeist.pm">
  <img src="./GH-banner.jpg">
</a>

# Zeitgeist: An Evolving Blockchain for Prediction Markets and Futarchy

![Rust](https://github.com/zeitgeistpm/zeitgeist/workflows/Rust/badge.svg)

<a href="https://t.me/zeitgeist_official">
  <img src="https://img.shields.io/badge/telegram-https%3A%2F%2Ft.me%2Fzeitgeist__official-blue" />
</a>

Zeitgeist is a decentralized network for creating, betting on, and resolving
prediction markets. The platform's native currency, the ZTG,
is used to sway the direction of the network, and as a means of last-call dispute
resolution. Additionally, Zeitgeist is a protocol for efficient trading of prediction
market shares and will one day become the backbone of the decentralized finance ecosystem
by allowing for traders to create complex financial contracts on virtually _anything_.

## Modules


- [authorized](./zrml/authorized) - Offers authorized resolution of disputes.
- [court](./zrml/court) - An implementation of a court mechanism used to resolve
  disputes in a decentralized fashion.
- [market-commons](./zrml/market-commons) - Contains common operations on markets
  that are used by multiple pallets.
- [orderbook-v1](./zrml/orderbook-v1) - A naive orderbook implementation that's
  only part of Zeitgeist's PoC. Will be replaced by a v2 orderbook that uses 0x-style
  hybrid on-chain and off-chain trading.
- [prediction-markets](./zrml/prediction-markets) - The core implementation of the
  prediction market logic for creating and resolving markets.
- [shares](./zrml/shares) - Implementation of tradable and transferrable shares of
  prediction market outcomes.
- [simple-disputes](./zrml-simple-disputes) - Simple disputes selects the last dispute
  after a predetermined amount of disputes as the canonical outcome.
- [swaps](./zrml/swaps) - An implementation of liquidity pools that allows any user
  to provide liquidity to the pool or swap assets in and out of the pool. The market
  maker that is traded against is either a Constant Function Market Maker (CFMM) or
  a Rikiddo Market Maker.
- [rikiddo](./zrml/rikiddo) - [Rikiddo][rikiddo] is our novel market scoring rule. It 
  is an extension of the [Liquidity Sensitive Market Scoring Rule (LS-LMSR)][ls-lmsr].
  it will be used by the market maker to determine the prices for swaps.

## How to Build Nodes

Zeitgeist node comes in two flavors, one for standalone self-contained execution and another for Kusama/Polkadot parachain integration.

To build the standalone version, simply point to the top directory of this project and type:

```bash
cargo build --release
```

For parachain, it is necessary to change the current directory to `node` and then select the appropriated compiler feature flag.

```
cargo build --features parachain --release
```

Optimized binaries are usually used for development (faster and smaller) but this behavior is optionally up to you. If desirable, it is also possible to run commands directly with `cargo run` instead of running `./target/(debug|release)/zeitgeist`.

### Using Docker

We publish the latest version to the [Docker Hub](https://hub.docker.com/r/zeitgeistpm/zeitgeist-node) 
that can be pulled and ran locally to connect to the network. In order to do this first make sure that
you have Docker installed locally, then type (or paste) the following commands in your terminal:

```
docker pull zeitgeistpm/zeitgeist-node
docker run zeitgeistpm/zeitgeist-node --chain battery_park
```

This will get you connected to the [Battery Park](https://docs.zeitgeist.pm/battery-park) testnet.



[ls-lmsr]: https://www.eecs.harvard.edu/cs286r/courses/fall12/papers/OPRS10.pdf
[rikiddo]: https://blog.zeitgeist.pm/introducing-zeitgeists-rikiddo-scoring-rule/
