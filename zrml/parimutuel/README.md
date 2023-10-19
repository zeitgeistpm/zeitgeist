# Parimutuel Module

The Parimutuel module implements a straightforward parimutuel market maker for
categorical markets.

## Overview

These are "losers pay winners" market makers: Any informant can bet any amount
at any time. Their bet amount goes into the _pot_ and they receive tokens which
represent their share of the pot. After the market is resolved, the entire pot
is distributed amongst those who wagered on the outcome that materialized,
proportional to what their share of the pot is.

Selling shares is not allowed in parimutuel markets (this may be subject to
change in the future). Parimutuel markets are only allowed to be used in
conjunction with categorical markets; scalar markets are not allowed.

If there is no bet on the winning outcome, all bets are cancelled and informants
can retrieve their funds from the pool, minus potential external fees paid to
other parties.

### Terminology

- _Collateral_: The currency type that backs the outcomes in the pool. This is
  also called the _base asset_ in other contexts.
- _External fees_: After taking swap fees, additional fees can be withdrawn from
  an informant's collateral. They might go to the chain's treasury or the market
  creator.
- _Pot_: The account that holds all the wagered funds.

### Notes

- There's a hard requirement that the existential deposit of
  `Asset::ParimutuelShares` be at least the existential deposit of the
  collateral used. This ensures that no whitelisting or other trickery is
  necessary to prevent the pot from getting dusted.
