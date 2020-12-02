# Zeitgeist: A Prediction Markets Blockchain and Governance Protocol

Zeitgeist is a decentralized network for creating, betting on, and resolving
prediction markets. The platform's native currency, the ZGE,
is used to sway the direction of the network, and as a means of last-call dispute
resolution. Additionally, Zeitgeist is a protocol for efficient trading of prediciton
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

## [Whitepaper](./zeitgeist.md)

## Roadmap

Zeitgeist is currently under development.

### [Battery Park](https://github.com/zeitgeistmarket/zeitgeist/milestone/1)

Release date: October 31, 2020

Battery Park is the Proof-of-Concept release of the Zeitgeist protocol
that implements the prediction markets core logic as well as a simple orderbook
for trading shares.

### [Unnamed](https://github.com/zeitgeistmarket/zeitgeist/milestone/2)

The next milestone after "Batter Park" is UNNAMED. It will integrate Balancer-style
automated market makers and liquidity mining as the core trading protocol.

