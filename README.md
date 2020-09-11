# Zeitgeist: A Prediction Markets Blockchain and Governance Protocol

Zeitgeist is a decentralized network for creating, betting on, and resolving
prediction markets. The platform's native currency, the Zeitgeist Token (ZGT),
is used to sway the direction of the network, and as a means of last-call dispute
resolution. Additionally, Zeitgeist is a protocol for efficient trading of prediciton
market shares and will one day become the backbone of the decentralized finance ecosystem
by allowing for traders to create complex financial contracts on virtually _anything_.

## Modules

- [orderbook-v1](./xrml/orderbook-v1) - A naive orderbook implementation that's
  only part of Zeitgeist's PoC. Will be replaced by a v2 orderbook that uses 0x-style
  hybrid on-chain and off-chain trading. Eventually will be replaced by an automated
  market maker based on Robin Hanson's Logarithmic Market Scoring Rule.
- [prediction-markets](./xrml/prediction-markets) - The core implementation of the
  prediction market logic for creating and resolving markets.
- [shares](./xrml/shares) - Implementation of tradable and transferrable shares of
  prediction market outcomes.

## [Whitepaper](./zeitgeist.md)
