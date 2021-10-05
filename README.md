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

- [orderbook-v1](./zrml/orderbook-v1) - A naive orderbook implementation that's
  only part of Zeitgeist's PoC. Will be replaced by a v2 orderbook that uses 0x-style
  hybrid on-chain and off-chain trading.
- [prediction-markets](./zrml/prediction-markets) - The core implementation of the
  prediction market logic for creating and resolving markets.
- [swaps](./zrml/swaps) - An implmenation of a constant function automated market maker
  (similar to Balancer) that allows for liqudity providing and trading of prediction
  market shares.
- [shares](./zrml/shares) - Implementation of tradable and transferrable shares of
  prediction market outcomes.
- [simple-disputes](./zrml-simple-disputes) - Simple disputes

## How to Build

To build the Zeitgeist node, simply point to the top directory of this project and type:

```bash
cargo build --bin zeitgeist --release
```

Optimized binaries are usually used for development (faster and smaller binaries) but this feature is up to you. If desirable, it is also possible to run commands directly with `cargo run -- commands` instead of `./target/(debug|release)/zeitgeist commands`.

### Using Docker

We publish the latest version to the [Docker Hub](https://hub.docker.com/r/zeitgeistpm/zeitgeist-node) 
that can be pulled and ran locally to connect to the network. In order to do this first make sure that
you have Docker installed locally, then type (or paste) the following commands in your terminal:

```
docker pull zeitgeistpm/zeitgeist-node
docker run zeitgeistpm/zeitgeist-node --chain battery_park
```

This will get you connected to the [Battery Park](https://docs.zeitgeist.pm/battery-park) testnet.
