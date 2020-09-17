# Zeitgesit Protocol

Zeitgeist is a decentralized network for creating, betting on, and resolving
prediction markets. The platform's native currency, the Zeitgeist Token (ZGT),
is used to sway the direction of the network, and as a means of last-call dispute
resolution. Additionally, Zeitgeist is a protocol for efficient trading of prediciton
market shares and will one day become the backbone of the decentralized finance ecosystem
by allowing for traders to create complex financial contracts on virtually _anything_.

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
  - [Uses of ZGT](#uses-of-zgt)
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

A prediction market is a type of market mechanism that is designed to solve the information
aggregation problem. In brief, the information aggregation problem can be described as
arising out of the need of an individual (known as the _aggregator_) wishing to obtain
a prediction on a particular variable. A number of inividuals known as the _informants_
each hold different views and information regarding the variable. A prediction market is
a mechanism to incentivize informants to report truthfully about their information and
therefore create a robust aggregation.

For example, let's say an individual would like to know what the predicted price of
Kusama ($KSM) will be at the end of year 2021. This individual can create a market
that asks "Will the price of KSM be more than $100 according to CoinGecko on December 31, 2020?".
This market has two sides. One side is the "Yes" share which will be worth $1 only if the outcome
finalizes to yes, and the other side is the "No" share which will be worth $1 only if the outcome
finalizes to no.
Traders can then speculate on the market outcome, using each of their indivudal knowledge and
information to inform their trades. For example, if Alice believes that KSM will be worth more
than $100 with a probability of 80% they are incentivized to buy "Yes" shares up to the
market price of $0.80.

-The source of the above information is David M. Pennock and Rahul Sami's Computational Aspect
of Prediction Markets in the Roughgarden-edited "Algorithmic Game Theory".-

Markets are noted in the efficient market hypothesis to be excellent aggregators of information.
It is specifically this property of markets that make prediction markets the most accurate
of all available tools to forecast future outcomes. Therefore, it is proposed by Hanson that
prediction markets can be used to make business decisions (i.e. Futarchy).

## How Zeitgeist Works

### Market Creation

### Trading

### Oracle Reports

### Dispute Resolution

### Market Finalizaton

## Governance

Zeitgeist is a blockchain network but also a DAO that oversees the health and
sustainability of this network. Along with the stakers, the primary maintainers of
the network, the ZGT holders themselves have the opportunity to evolve the protocol.
In times of crisis, the ZGT holders will be the party of last resort to avert danger
and protect the protocol from collapsing. For these reasons the ZGT is a governance token
of the network. Below we detail the governance mechanism of the Zeitgeist DAO.

Zeitgeist governance is divided into five distinct chambers, each with checks and balances
on the other chambers.

### Executive Council

There may be a large incentive for a reporter to buy the opposing share of a market
then report fraudelently. In most cases, the native dispute system will incentivize and
fairly resolve the fraudulent behaviour before a payout is made. However, in some cases
it may be that there is such an incentive that a significant amount of ZGT holders are
reporting fraudelently. In Augur this is resolved through a protocol fork - a messy process
that requires all REP holders to migrate their tokens to what they believe is the right
side of history within a limited time at the risk of losing all their REP completely. 
Instead of using a forking procedure, Zeitgeist solves this problem more elegantly through
use of an elected on-chain body known as the _Executive Council_ that can call a public
vote for all ZGT holders to decide the final outcome of a market.

### Advisory Committee

When someone creates a market on Zeitgeist they have two available options for the size
of the deposit they are willing to make using ZGT. One option is to place a large deposit
for example 10 ZGT and have their market be activated right away. The large deposit is
there as assurance to the network that the market will not be resolved as an invalid
market. This is because invalid markets are overall harmful for an ecosystem (see the 
section on [invalid markets](#invalid-markets)).

### Democracy

### Treasury

Zeitgeist will have an on-chain treasury which can be permissionlessly requested for funds
and will be paid out through majority acceptance of the Executive Council. 

### Advisory Committee

### Public Reporters / Court

Sometimes it may be desirable for a market creator to not specify a particular oracle source,
and rather just a resolution source. In cases like this, the default is to draw an oracle
from the public reporters pool. The public reporters pool can be joined by any ZGT holder.
They stake their ZGT and enter into the pool. Anytime there is a market that needs resolution
the first reporter from the public reporter pool can make a report. They then have a cooldown
for making another public report for a number which is a function of how many times in the past
they have made public reporting.

## Token Economics

### Uses of ZGT

### Emission Schedule

### Fair Launch

## Roadmap

We aim to distribute ZGT in as fair and egalitarian way as possible, while still
acknowledging the necessary steps to bring the network to fruition. Because we have
such a great foundation laid down for us by the Augur protocol, and because the Augur
community has demonstrated itself knowledgable about prediction markets in general we
plan to airdrop 5% of the initial allocation of ZGT to REP holders.

- 10% Initial Founding Team
-  5% Airdrop to REP holders that signal.
- 10% ETH lockdrop.
-  5% Early investors.
-  5% Users of the testnet.
- 25% Initial Uniswap Offering
- 40% Reserves

## Applications and Use Cases

Finally we conclude with some high level descriptions of use cases and business that can
be built atop the Zeitgesit blockchain network.

### E-Sports

### Politics

### Cryptocurrency

### Insurance

A next generation decentralized insurance business can be built atop the Zeitgeist protocol.
This is because insurance is essientially a way for an individual to de-risk themselves
from a certain event happening. Prediction markets are perfect for this as the individual
just needs to purchase shares in the event that they are hoping will not take place.

For example, if a person wants to insure themselves against a DeFi protocol failing and
losing all of their money what they can do is the following:

1) They would purchase shares in a "Will DeFi protocol X collapse?" for "Yes" equal to the
probability...

### Startups

### Futarchy

Futarchy is the use of prediction markets to guide decision making. When prediction markets are
used in Futarchy, they are sometimes referred to as _decision markets_ instead. However, the two
are not mutually exclusive and any prediction market can double as a decision market if there is
an entity that is bound to making its decision based on it.

<!-- Everything below this line is draft content to be integrated into the real content above -->
<!--

## Protocol Description

TODO: merge into the how it works section

### Market Creation

### Shares

Shares are generated and destroyed as a complete set directly from the market. When shares
are generated, the amount held in the reserve pool of that market will be increased. When shares
are destroyed, the supply will decreased and the reserve pool will be used to pay out ZGT in 
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
the Zeitgeist chain and so the ZGT would cease to have inflation. This would mean
that for the time that Zeitgeist runs as its own sovereign chain will be the only
time that new tokens would be created. Additionally, if Zeitgeist were to return
to being a sovereign chain after a bout of being a parachain - it would required
the work of validators once again and therefore the token would again be inflated.

## Native Rebalancing Stablecoin

Since that native token of Zeitgeist, the ZGT is used as a governance token and a
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

## Usages of the ZGT

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
REP a security). Zeitgeist will have its entire governance controlled by the ZGT holders
including the governance ability of not only resolving markets, but electing the advisory
committee, the oracle committee, and the upgrade roadmap.

## Logarithmic Market Scoring Rule

## Problems with the Continuous Double Auction / Orderbook

The thin market problem. Liquidity must be bootstrapped in some way. Also the double
coincidence of wants.

# Court

If a market goes through the entire court process and ultimately reaches the conclusion of
"Invalid" then all of the shares that were used to participate in the market will take a 10%
haircut and the ZGT will be redistributed to those who participated in the dispute process.

[Hanson,LMSR]: https://mason.gmu.edu/~rhanson/mktscore.pdf

-->
