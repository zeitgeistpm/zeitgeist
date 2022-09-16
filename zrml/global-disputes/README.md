# Global Disputes

A module for setting one out of multiple outcomes with the most locked native tokens as the canonical outcome.

## Overview

This is the default process when a dispute mechanism (e. g. Court) fails to resolve. In the zeitgeist ecosystem this grants the ability to lock native tokens by voting on one of multiple outcomes to determine the canonical outcome on which the market finally resolves.

`outcome_sum` - The actual amount of native tokens for one outcome, which is used to calculate the outcome with the most locked native tokens.

## Interface

### Dispatches

#### Public Dispatches

- `add_vote_outcome` - After a global dispute is started and before it's finished, everyone can add a voting outcome for a constant fee. If the outcome already exists, an error is returned. There can only be one owner of the outcome.
- `vote_on_outcome` - Vote on existing voting outcomes by locking native tokens. Fails if the global dispute has not started or has already finished.
- `unlock_vote_balance` - All locked native tokens due to voting of finished global disputes get returned.
- `reward_outcome_owner` - The collected fees for adding voting outcomes are rewarded to the winning outcome owner/s (only one owner per outcome exists when using `add_vote_outcome`, but multiple owners can exist when calling `push_voting_outcome`).

#### Private Pallet API

- `push_voting_outcome` - This is meant to be called to start a global dispute. Add an outcome (with initial vote balance) to the voting outcomes. The `outcome_sum` of exact same outcomes is added. There can be multiple owners of one same outcome.
- `get_voting_winner` - This is meant to be called to finish a global dispute. Determine the outcome with the highest `outcome_sum` as the winner.
- `is_started` - Check if the global dispute started already.
