<a href="https://zeitgeist.pm">
  <img src="./GH-banner.svg" width="800">
</a>

# Zeitgeist: An Evolving Blockchain for Prediction Markets and Futarchy

![Rust](https://github.com/zeitgeistpm/zeitgeist/actions/workflows/rust.yml/badge.svg)
[![Codecov](https://codecov.io/gh/zeitgeistpm/zeitgeist/branch/main/graph/badge.svg)](https://codecov.io/gh/zeitgeistpm/zeitgeist)
[![Discord](https://img.shields.io/badge/-Zeitgeist-blue?logo=discord&logoColor=ffffff&style=flat)](https://discord.gg/XhAcFWYUej)
[![Telegram](https://img.shields.io/badge/-zeitgeist_official-blue?logo=telegram&style=flat)](https://t.me/zeitgeist_official)
[![X](https://img.shields.io/badge/-zeitgeistpm-blue?logo=X&style=flat)](https://twitter.com/zeitgeistpm)

Zeitgeist is a decentralized network for creating, betting on, and resolving
prediction markets, allowing traders to create complex financial contracts on
virtually _anything_. The platform's native currency ZTG is used to sway the
direction of the network, and as a means of last-call dispute resolution in the
decentralized court.

## Modules

- [authorized](./zrml/authorized) - Offers authorized resolution of disputes.
- [court](./zrml/court) - An implementation of a court mechanism used to resolve
  disputes in a decentralized fashion.
- [global-disputes](./zrml-global-disputes) - Global disputes sets one out of
  multiple outcomes with the most locked ZTG tokens as the canonical outcome.
  This is the default process if a dispute mechanism fails to resolve.
- [macros](./macros) - Contains macros shared by the other modules.
- [market-commons](./zrml/market-commons) - Contains common operations on
  markets that are used by multiple pallets.
- [neo-swaps](./zrml/neo-swaps) - An implementation of the Logarithmic Market
  Scoring Rule as constant function market maker, tailor-made for decentralized
  combinatorial markets and futarchy.
- [orderbook](./zrml/orderbook) - An order book implementation.
- [parimutuel](./zrml/parimutuel) - A straightforward parimutuel market maker
  for categorical markets.
- [prediction-markets](./zrml/prediction-markets) - The core implementation of
  the prediction market logic for creating and resolving markets.
- [swaps](./zrml/swaps) - An implementation of the Balancer CFMM that allows any
  user to create pools, provide liquidity or swap assets.
- [primitives](./zrml/primitives) - Contains custom and common types, traits and
  constants.
- [rikiddo](./zrml/rikiddo) - The module contains a completely modular
  implementation of our novel market maker [Rikiddo][rikiddo]. It also offers a
  pallet that other pallets can use to utilize the Rikiddo market maker. Rikiddo
  can be used by the automated market maker to determine swap prices.

## How to Build and Run a Zeitgeist Node

Zeitgeist node comes in two flavors, one for standalone self-contained execution
and another for Kusama/Polkadot parachain integration.

To build the standalone version for testing, simply point to the top directory
of this project and type:

```bash
cargo build --release
```

The standalone version uses the runtime defined for Zeitgeist's testnet _Battery
Station_ in [runtimes/battery-station](runtimes/battery-station) and is run in
`--dev` mode by default.

To build the parachain version, execute the following command:

```
cargo build --features parachain --release
```

By default, the parachain version will connect to the Zeitgeist main network,
which launched as a parachain of Kusama and has since been migrated to Polkadot.
The runtime of the main network is defined in
[runtimes/zeitgeist](runtimes/zeitgeist).

To connect to Zeitgeist's testnet Battery Station, which runs as a parachain of
Rococo, run:

```
cargo run --features parachain --release -- --chain=battery-station
```

Optimized binaries (`--release`) are usually used for production (faster and
smaller), but this behavior is optional and up to you.

### Using Docker

We publish the latest standalone and parachain version to the [Docker
Hub][zg-docker-hub], from where it can be pulled and ran locally to connect to
the network with relatively low effort and high compatibility. In order to fetch
the latest docker image, ensure you have Docker installed locally, then type (or
paste) the following commands in your terminal.

For parachain Zeitgeist node:

```
docker pull zeitgeistpm/zeitgeist-node-parachain
```

For standalone, non-parachain Zeitgeist node:

```
docker pull zeitgeistpm/zeitgeist-node
```

To connect your Zeitgeist parachain node using Docker, follow the tutorial at
our [documentation site][bs-docs].

Alternatively you can run a non-parachain node, which is usually only necessary
for testing purposes, by executing the following command:

```
docker run zeitgeistpm/zeitgeist-node -- <node-options-and-flags>
```

[bs-docs]: https://docs.zeitgeist.pm/docs/basic/battery-station
[ls-lmsr]: https://www.eecs.harvard.edu/cs286r/courses/fall12/papers/OPRS10.pdf
[rikiddo]:
  https://blog.zeitgeist.pm/introducing-zeitgeists-rikiddo-scoring-rule/
[battery-station]: https://blog.zeitgeist.pm/zeitgeist-beta/
[zg-docker-hub]: https://hub.docker.com/r/zeitgeistpm/zeitgeist-node
