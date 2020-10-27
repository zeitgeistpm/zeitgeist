# Zeitgesit Protocol

Zeitgeist is an self-evolving blockchain for prediction markets with a governance
protocol. The core functions of the Zeitgeist blockcahin include methods for
creating, betting on, and resolving prediction markets, although this is expected
to be expanded in the future.

A prediction market, as Zeitgeist implements them, is a permissionless
speculative market that allows anyone to pose a question about a future
occurrance and other individuals to trade on the outcomes. Zeitgeist's native
currency, the ZGE, is used as bonds for creating and resolving markets, as well
as the default trading pair for all markets. Additionally, Zeitgeist implements
a robust governance protocol around its core functionality that allows ZGE
holders to participate, vote, and ultimately sway the direction of the network
in perpetuity.

An area of exploration for the Zeitgeist project is futarchy.

## Table of Contents

- [What are Prediction Markets?](#what-are-prediction-markets)
- [How Zeitgeist Works](#how-zeitgeist-works)
  - [Market Creation](#market-creation)
  - [Trading](#trading)
  - [Oracle Reports](#oracle-reports)
  - [Dispute Resolution](#dispute-resolution)
  - [Market Finalization](#market-finalization)
- [Governance](#governance)
  - [Executive Council](#executive-council)
  - [Advisory Committee](#advisory-committee)
  - [Democracy](#democracy)
  - [Treasury](#treasury)
  - [Court](#court)
- [Token Economics](#token-economics)
  - [Uses of ZGE](#uses-of-ZGE)
  - [Emission Schedule](#emission-schedule)
  - [Fair Launch](#fair-launch)
- [Roadmap]
- [Applications](#applications-and-use-cases)
  - [E-Sports](#e-sports)
  - [Politics](#politics)
  - [Cryptocurrency](#cryptocurrency)
  - [Insurance](#insurance)
  - [Startups](#startups)
  - [Futarchy](#futarchy)

## What are Prediction Markets?

A prediction market is a type of market mechanism that is designed to solve the
information aggregation problem. In brief, the information aggregation problem
can be described as arising out of the need of an individual (known as the
_aggregator_) wishing to obtain a prediction on a particular variable. A number
of inividuals known as the _informants_ each hold different views and
information regarding the variable. A prediction market is a mechanism to
incentivize informants to report truthfully about their information and
therefore create a robust aggregation and accurate signal of the predicted
outcome.

For example, let's say an individual would like to discover the predicted price
of Kusama's native token, the KSM, in the year 2021. Before this individual,
known as the aggregator, can create this market they must form their question in
a precise way. Therefore, when they create the market they would pose the
questions as "Will the price of KSM be more than \$100 according to the
CoinGecko API on July 20, 2021 at 23:59 UTC?". This form of the question is much
more precise than "Will KSM be worth a lot in 2021?".

Once the market is created, in the most common scenario there are two tradable
sides: "Yes" or "No". A market maker can generate 1 "Yes" share and 1 "No" share
in exchange for 1 of the base currency (ZGE). If the market is later found to
resolve to "Yes" then the "Yes" shares will be redeemable for 1 ZGE. Vice versa,
if the market later resolves to "No", the "No" shares will be worth 1 ZGE. The
losing share is not redeemable for ZGE.

Traders can speculate and bet based on their individual information and belief
on the market outcome. While trading, the price of each share reflects the
expected probability that the market will resolve on that outcome. For example,
if "Yes" shares in the market are trading at
$0.60 while "No" shares
are trading at $0.39 this displays an expected
probability of the market resolving "Yes" at 60%. The difference between the
price of the "Yes" share and the price of the "No" share reflects the spread in
the market, which can be arbitraged by market makers. If a trader believes the
probability of KSM being worth more than
$100 at the specified date is actually 80%, they are economically
incentivized to buy "Yes" shares up to the price of $0.80.
Similarly, they could sell "No" shares down to the price of \$0.20.

<!-- The source of the above information is David M. Pennock and Rahul Sami's Computational Aspect
of Prediction Markets in the Roughgarden-edited "Algorithmic Game Theory". -->

Markets are noted in the efficient market hypothesis to be excellent aggregators
of information. It is specifically this property of markets that make prediction
markets the most accurate of all available tools to forecast future outcomes.
Therefore, it is proposed by Robin Hanson that prediction markets can be used to
make business decisions (i.e. [Futarchy](#futarchy)).

## How Zeitgeist Works

We will now discuss from a high level overview. We will delve into some detail
when necessary, but leave many of the particulars related to governance for the
expansion in the next section.

### Market Creation

In order for anything exciting to happen on Zeitgeist, the first step is the
creation of a prediction market. Since Zeitgeist is itself a blockchain, based
on the core priniciples or peer-to-peer and decentralization, it is
permissionless for anyone to create a new market for any question they would
like to pose. Prediction markets perform better when the question that is posed
is precise in the variable that it tests as well as the resolution details. In
the past we have seen that sometimes the way a question is posed can have
significant impact in the ending resolution of a market. [TODO: cite the famous
example]. As in numerous precedents on Augur, it is possible for malicious
actors to pose knowingly bad markets in order to try to con honest traders.

Zeitgeist implements two methods for a market: 1) a full permissionless model
where the market can be created, and 2) a semi-permissioned model where a market
must pass approval from the advisory commitee (a chamber of Zeitgeist
goverance). One difference between these two methods for the market creator is
that the first option (the permissionless option) requires a larger bond than
the permissioned option. This bond is held for the entire duration of the market
and can be slashed if eventuall the Zeitgeist Court decides that the market was
invalid. Alternatively, a person may place a smaller bond if they wish to wait
until the market first gets approval from the Advisory Committee. When the
advisory committee approves a market, the bond is returned right away to the
creator, but if the market is rejected as invalid then the bond will be burned.

When a market has been created, depending on what kind of bond was placed on it,
it either moves straight to being active (permissionless bond) or it moves to
the advisory queue (permissioned bond), where it waits to be approved.

<!-- Add more information on the parameters passed to market creation. -->

#### Market Types

#### Market Parameters

<!-- Add information about IPFS -->

### Trading

Once a market is in its active phase, it is open for traders to start buying and
selling shares of the outcomes.

In the prediction market literature there are typically two mechanisms which are
used for trading on a prediction market. The first method, a continuous double
auction (CDA), is a design that is familiar to most traders as a two-sided
orderbook. The second method, which has recently gained steam in the
decentralized finance (DeFi) community, is known as an automated market maker
(AMM). Zeitgeist will implement both types of trading for its markets and may
implement more optimized or new trading designs in the future. Since Zeitgeist
can evolve with through it governance mechanisms, adding new trading features
over time will be possible. For more information on the implementation of
different trading options please see the milestone presented in the
[roadmap](#roadmap).

#### Continuous Double Actions (Orderbook)

#### LS-LSMR

### Oracle Reports

Once a market reaches its ending date that was set when it created it will be
moved automatically to a `Closed` state. In the `Closed` state the market no
longer allows any new shares to be generated. However, trading for the already
existing shares can continue on indefinitely.

The market must now be resolved by having its designated `Market.oracle` report
on its outcome. The designated oracle has 24 hours to report. If the oracle
fails to report, then the `OraceleBond` will be slashed from the market creator
and the market will be reportable by anyone of the Public Reporter Pool, a group
of staked actors who are rewarded with ZGE and are allowed to place bonds for
reporting on any market that hasn't been reported on yet. The public reporter
will place a bond in ZGE with their report.

After the report is registered on chain, there is another 48 hours during which
the market can be disputed. If a market is disputed, the actor who called the
dispute must place a `DisputeBond` behind the claim. The market will then be
managed through the decentralized court's dispute resolution mechanism.

### Dispute Resolution

### Market Finalizaton

## Governance

Zeitgeist is a blockchain network but also a DAO that oversees the health and
sustainability of this network. Along with the stakers, the primary maintainers
of the network, the ZGE holders themselves have the opportunity to evolve the
protocol. In times of crisis, the ZGE holders will be the party of last resort
to avert danger and protect the protocol from collapsing. For these reasons the
ZGE is a governance token of the network. Below we detail the governance
mechanism of the Zeitgeist DAO.

Zeitgeist governance is divided into five distinct chambers, each with checks
and balances on the other chambers.

### Executive Council

There may be a large incentive for a reporter to buy the opposing share of a
market then report fraudelently. In most cases, the native dispute system will
incentivize and fairly resolve the fraudulent behaviour before a payout is made.
However, in some cases it may be that there is such an incentive that a
significant amount of ZGE holders are reporting fraudelently. In Augur this is
resolved through a protocol fork - a messy process that requires all REP holders
to migrate their tokens to what they believe is the right side of history within
a limited time at the risk of losing all their REP completely. Instead of using
a forking procedure, Zeitgeist solves this problem more elegantly through use of
an elected on-chain body known as the _Executive Council_ that can call a public
vote for all ZGE holders to decide the final outcome of a market.

### Advisory Committee

When someone creates a market on Zeitgeist they have two available options for
the size of the deposit they are willing to make using ZGE. One option is to
place a large deposit for example 10 ZGE and have their market be activated
right away. The large deposit is there as assurance to the network that the
market will not be resolved as an invalid market. This is because invalid
markets are overall harmful for an ecosystem (see the section on
[invalid markets](#invalid-markets)).

### Democracy

### Treasury

Zeitgeist will have an on-chain treasury which can be permissionlessly requested
for funds and will be paid out through majority acceptance of the Executive
Council.

### Advisory Committee

### Public Reporters / Court

Sometimes it may be desirable for a market creator to not specify a particular
oracle source, and rather just a resolution source. In cases like this, the
default is to draw an oracle from the public reporters pool. The public
reporters pool can be joined by any ZGE holder. They stake their ZGE and enter
into the pool. Anytime there is a market that needs resolution the first
reporter from the public reporter pool can make a report. They then have a
cooldown for making another public report for a number which is a function of
how many times in the past they have made public reporting.

## Token Economics

### Uses of ZGE

### Emission Schedule

### Fair Launch

## Roadmap

We aim to distribute ZGE in as fair and egalitarian way as possible, while still
acknowledging the necessary steps to bring the network to fruition. Because we
have such a great foundation laid down for us by the Augur protocol, and because
the Augur community has demonstrated itself knowledgable about prediction
markets in general we plan to airdrop 5% of the initial allocation of ZGE to REP
holders.

- 10% Initial Founding Team
- 5% Airdrop to REP holders that signal.
- 10% ETH lockdrop.
- 5% Early investors.
- 5% Users of the testnet.
- 25% Initial Uniswap Offering
- 40% Reserves

### PoC-1 "Battery Park"

- Battery Park testnet.
- Trading uses naive orderbooks.
- Polkadot-JS Apps integration.

### PoC-2 "Alphabet City"

- Alphabet City testnet.
- Next version of orderbooks.
- The ability to use a LS-LMSR AMM.

### Mainnet

- The ZGE token launches.
- Mobile application.


## Applications and Use Cases

Finally we conclude with some high level descriptions of use cases and business
that can be built atop the Zeitgesit blockchain network.

### E-Sports

### Politics

### Cryptocurrency

### Insurance

A next generation decentralized insurance business can be built atop the
Zeitgeist protocol. This is because insurance is essientially a way for an
individual to de-risk themselves from a certain event happening. Prediction
markets are perfect for this as the individual just needs to purchase shares in
the event that they are hoping will not take place.

For example, if a person wants to insure themselves against a DeFi protocol
failing and losing all of their money what they can do is the following:

1. They would purchase shares in a "Will DeFi protocol X collapse?" for "Yes"
   equal to the probability...

### Startups

### Futarchy

Futarchy is the use of prediction markets to guide decision making. When
prediction markets are used in Futarchy, they are sometimes referred to as
_decision markets_ instead. However, the two are not mutually exclusive and any
prediction market can double as a decision market if there is an entity that is
bound to making its decision based on it.

<!-- Everything below this line is draft content to be integrated into the real content above -->
<!--

## Protocol Description

TODO: merge into the how it works section

### Market Creation

### Shares

Shares are generated and destroyed as a complete set directly from the market. When shares
are generated, the amount held in the reserve pool of that market will be increased. When shares
are destroyed, the supply will decreased and the reserve pool will be used to pay out ZGE in
accordance to the amount of shares burned.

Shares can be transferred freely and traded among users of the network. They are identified by
a hash of the market identifier and the outcome identifier. For example, in a binary market
where there are 3 possible outcomes: Yes, No, or Invalid. The share identifier for the market
would be `hash(market_id, 0)` for Yes shares, `hash(market_id, 1)` for No shares, and `hash(market_id, 2)`
for Invalid shares.

## Trading

Zeitgeist has two built-in methods to trade shares. These methods are a hybrid on- and off-chain
orderbook and a Uniswap-style Automated Market Maker (AMM). For those familiar with the 0x-style
orderbooks in the Ethereum ecosystem, Zeitgeist takes a nearly identical approach. Namely, operators
(known in parlance as relayers) will keep the data of orders and only the hash of orders are published
on-chain. This allows the chain to scale very well as it means all orders require only a constant size
hash to be published on-chain.

### Reporting

### Forkless Resolutions

### Invalid Markets

TODO: Write about invalid markets.

## Substrate Pallets

- Prediction Markets
- Orderbook
- Shares
- AMM (LMSR-based)

## Zeitgeist's Transition to a Parachain

Zeitgeist plans to transition to a parachain on the Kusama or Polkadot networks.
When the transition takes place, validators will no longer be needed to secure
the Zeitgeist chain and so the ZGE would cease to have inflation. This would mean
that for the time that Zeitgeist runs as its own sovereign chain will be the only
time that new tokens would be created. Additionally, if Zeitgeist were to return
to being a sovereign chain after a bout of being a parachain - it would required
the work of validators once again and therefore the token would again be inflated.

## Native Rebalancing Stablecoin

Since that native token of Zeitgeist, the ZGE is used as a governance token and a
utility token over the platform to gracefully resolve markets, it may not be the first
choice for the token that is used to participate in markets. Without a doubt, the ideal
choice for a token to place orders on a prediction market will be a stablecoin.

Once Zeitgeist integrates with the wider Polkadot ecosystem, there will be many options
for the stablecoin to use from projects such as Acala. However, while Zeitgeist remains
a sovereign chain there will need to be some stablecoin.

Option 1 -
Since Zeitgeist is primarly a prediction market platform, it will employ a native prediction
market that will use a Schellingcoin-like mechanism to rebalance.

Option 2 -
Zeitgeist will use a simple collateral-based stablecoin.


# Executive Summary

## Usages of the ZGE

- To place bonds on market creation.
- To participate as part of the advisory committee or voting for members of the advisory
  committee.
- As the native currency on which the outcomes of prediction markets will be paid.
- As a staking currency for validators and nominators of the network.

# Appendix A: Differences between Augur and Zeitgeist

## Cost and Scalability

While Augur is built atop a suite of smart contracts on the Etherum 1.x chain, there is
a significant gas cost to using the main interface. (Include some latest stats here). Since
Zeitgeist has been abstracted to its own sovereign chain, it will be able to scale much
more easily since it does not need to share block bandwidth with other non-related
applications. Furthermore, by using the Substrate framework the protocol is able to
be optimized on a native base chain layer level which is simply not possible for Augur.

## Optimization

## Governance

While Augur is governed by the REP holders ultimately, its development direction is still
mostly controlled by the centralized entity of the Forecast Foundation (see coinbase rating
REP a security). Zeitgeist will have its entire governance controlled by the ZGE holders
including the governance ability of not only resolving markets, but electing the advisory
committee, the oracle committee, and the upgrade roadmap.

## Logarithmic Market Scoring Rule

## Problems with the Continuous Double Auction / Orderbook

The thin market problem. Liquidity must be bootstrapped in some way. Also the double
coincidence of wants.

# Court

If a market goes through the entire court process and ultimately reaches the conclusion of
"Invalid" then all of the shares that were used to participate in the market will take a 10%
haircut and the ZGE will be redistributed to those who participated in the dispute process.

[Hanson,LMSR]: https://mason.gmu.edu/~rhanson/mktscore.pdf

-->
