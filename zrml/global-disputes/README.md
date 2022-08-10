# Global Disputes

A module for setting one out of multiple outcomes with the most locked native tokens as the canonical outcome.

## Overview

This is the default process if a dispute mechanism (e. g. Court) fails to resolve, because the `MaxDisputes` amount of disputes is reached. In the zeitgeist ecosystem this grants the ability to lock native tokens in a voting mechanism on multiple outcomes for determining the outcome on which the market resolves.

`outcome_sum` - The actual amount of native tokens for one outcome, which is used to calculate the outcome with the most locked native tokens.

## Interface

### Dispatches

#### Public Dispatches

- `add_vote_outcome` - After a global dispute is started and before it's finished, everyone can add a voting outcome for a constant fee. If the outcome already exists, an error is returned. There can only be one owner of the outcome.
- `vote_on_outcome` - After a global dispute is started and before it's finished, everyone can vote with locking native tokens on existing voting outcomes. The `outcome_sum` is increased only for the surplus. This means that multiple votes can be cast on outcomes. As long as the voting amount increases, only the increased difference amount is included in the `outcome_sum`. This method allows to store less on the chain, but still have the opportunity to vote on multiple outcomes without the attack vector to vote with the exact same locked tokens multiple times.
- `unlock_vote_balance` - All locked native tokens of finished global disputes get returned.
- `reward_outcome_owner` - The collected fees for adding voting outcomes are rewarded to the outcome owner/s (only one owner per outcome exists when using `add_vote_outcome`, but multiple owners can exist when calling `push_voting_outcome`).

#### Private Pallet API

- `push_voting_outcome` - This is meant to be called to start a global dispute. Add an outcome (with initial vote balance) to the voting outcomes. The `outcome_sum` of exact same outcomes is added. There can be multiple owners of one same outcome.
- `get_voting_winner` - This is meant to be called to finish a global dispute. Determine the outcome with the highest `outcome_sum` as the winner.
