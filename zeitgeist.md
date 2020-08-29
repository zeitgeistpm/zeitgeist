# Zeitgesit Protocol

Zeitgeist is a decentralized network for creating, betting on, and resolving
prediction markets. The platform's native currency the Zeitgesit Governance Token (XZG)
is used to decide the direction of the network and as a means of last-call dispute
resolution. Additionally, Zeitgeist is a protocol for efficient trading prediciton
market shares and will one day become the backbone of the decentralized finance eocsystem
by allowing for traders to create complex financial contracts on _anything_.

## Zeitgeist Governance / The Zeitgeist DAO

Zeitgeist is a blockchain network but also a DAO that oversees the health and
sustainability of this network. Along with the stakers, the primary maintainers of
the network, the XZG holders themselves have the opportunity to evolve the protocol.
In times of crisis, the XZG holders will be the party of last resort to avert danger
and protect the protocol from collapsing. For these reasons the XZG is a governance token
of the network. Below we detail the governance mechanism of the Zeitgeist DAO.

### Advisory Committee

When someone creates a market on Zeitgeist they have two available options for the size
of the deposit they are willing to make using XZG. One option is to place a large deposit
for example 10 XZG and have their market be activated right away. The large deposit is
there as assurance to the network that the market will not be resolved as an invalid
market. This is because invalid markets are overall harmful for an ecosystem (see the 
section on [invalid markets](#invalid-markets)).

### Arbitration Council

There may be a large incentive for a reporter to buy the opposing share of a market
then report fraudelently. In most cases, the native dispute system will incentivize and
fairly resolve the fraudulent behaviour before a payout is made. However, in some cases
it may be that there is such an incentive that a significant amount of XZG holders are
reporting fraudelently. In Augur this is resolved through a protocol fork - a messy process
that requires all REP holders to migrate their tokens to what they believe is the right
side of history within a limited time at the risk of losing all their REP completely. 
Instead of using a forking procedure, Zeitgeist solves this problem more elegantly through
use of an elected on-chain body known as the _Arbitration Council_ that can call a public
vote for all XZG holders to decide the final outcome of a market.

### Treasury

Zeitgeist will have an on-chain treasury which can be permissionlessly requested for funds
and will be paid out through majority acceptance of the Arbitration Council.

### Public Reporters

Sometimes it may be desirable for a market creator to not specify a particular oracle source,
and rather just a resolution source. In cases like this, the default is to draw an oracle
from the public reporters pool. The public reporters pool can be joined by any XZG holder.
They stake their XZG and enter into the pool. Anytime there is a market that needs resolution
the first reporter from the public reporter pool can make a report. They then have a cooldown
for making another public report for a number which is a function of how many times in the past
they have made public reporting.

## Protocol Description

### Market Creation

### Shares

Shares are generated and destroyed as a complete set directly from the market. When shares
are generated, the amount held in the reserve pool of that market will be increased. When shares
are destroyed, the supply will decreased and the reserve pool will be used to pay out XZG in 
accordance to the amount of shares burned.

Shares can be transferred freely and traded among users of the network. They are identified by
a hash of the market identifier and the outcome identifier. For example, in a binary market
where there are 3 possible outcomes: Yes, No, or Invalid. The share identifier for the market
would be `hash(market_id, 0)` for Yes shares, `hash(market_id, 1)` for No shares, and `hash(market_id, 2)`
for Invalid shares.


### Reporting

### Forkless Resolutions

### Invalid Markets

TODO: Write about invalid markets.

## Roadmap

We aim to distribute XZG in as fair and egalitarian way as possible, while still
acknowledging the necessary steps to bring the network to fruition. Because we have
such a great foundation laid down for us by the Augur protocol, and because the Augur
community has demonstrated itself knowledgable about prediction markets in general we
plan to airdrop 5% of the initial allocation of XZG to REP holders.

- 10% Initial Founding Team
-  5% Airdrop to REP holders that signal.
- 10% ETH lockdrop.
-  5% Early investors.
-  5% Users of the testnet.
- 25% Initial Uniswap Offering
- 40% Reserves

## Substrate Pallets

- Prediction Markets
- Swaps     

## Zeitgeist's Transition to a Parachain

Zeitgeist plans to transition to a parachain on the Kusama or Polkadot networks.
When the transition takes place, validators will no longer be needed to secure
the Zeitgeist chain and so the XZG would cease to have inflation. This would mean
that for the time that Zeitgeist runs as its own sovereign chain will be the only
time that new tokens would be created. Additionally, if Zeitgeist were to return
to being a sovereign chain after a bout of being a parachain - it would required
the work of validators once again and therefore the token would again be inflated.

## Native Rebalancing Stablecoin

Since that native token of Zeitgeist, the XZG is used as a governance token and a
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

## Use Cases

Finally we conclude with some high level descriptions of use cases and business that can
be built atop the Zeitgesit blockchain network.

### E-Sports Betting

### Insurance

A next generation decentralized insurance business can be built atop the Zeitgeist protocol.
This is because insurance is essientially a way for an individual to de-risk themselves
from a certain event happening. Prediction markets are perfect for this as the individual
just needs to purchase shares in the event that they are hoping will not take place.

For example, if a person wants to insure themselves against a DeFi protocol failing and
losing all of their money what they can do is the following:

1) They would purchase shares in a "Will DeFi protocol X collapse?" for "Yes" equal to the
probability...

### Futarchy

# Executive Summary

## Usages of the XZG

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
mostly controlled by the centralized entity of the forecast foundation (see coinbase rating
REP a security). Zeitgeist will have its entire governance controlled by the XZG holders
including the governance ability of not only resolving markets, but electing the advisory
committee, the oracle committee, and the upgrade roadmap.