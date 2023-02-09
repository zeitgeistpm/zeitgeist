# Global Disputes

A module for setting one out of multiple outcomes with the most locked native
tokens as the canonical outcome.

## Overview

This is the default process when a dispute mechanism (e. g. Court) fails to
resolve. In the zeitgeist ecosystem this grants the ability to lock native
tokens by voting on one of multiple outcomes to determine the canonical outcome
on which the market finally resolves.

## Terminology

- `outcome_sum` - The actual amount of native tokens for one outcome, which is
  used to calculate the outcome with the most locked native tokens.

## Interface

### Dispatches

#### Public Dispatches

- `add_vote_outcome` - Add voting outcome to a global dispute in exchange for a
  constant fee. Errors if the voting outcome already exists or if the global
  dispute has not started or has already finished.
- `vote_on_outcome` - Vote on existing voting outcomes by locking native tokens.
  Fails if the global dispute has not started or has already finished.
- `unlock_vote_balance` - Return all locked native tokens in a global dispute.
  If the global dispute is not concluded yet the lock remains.
- `purge_outcomes` - Purge all outcomes to allow the winning outcome owner(s) to
  get their reward. Fails if the global dispute is not concluded yet.
- `reward_outcome_owner` - Reward the collected fees to the owner(s) of a voting
  outcome. Fails if not all outcomes are already purged.
- `refund_vote_fees` - Return all vote funds and fees, when a global dispute was
  destroyed.

#### Private Pallet API

- `push_vote_outcome` - Add an initial voting outcome and vote on it with
  `initial_vote_balance`.
- `get_voting_outcome_info` - Get the information (`outcome_sum`, `owners`) of a
  voting outcome.
- `determine_voting_winner` - Determine the canonical voting outcome based on
  total locked tokens.
- `does_exist` - Check if the global dispute does already exist.
- `is_active` - Check if the global dispute is active to get votes
  (`vote_on_outcome`) and allow the addition of new voting outcomes with
  `add_vote_outcome`.
- `start_global_dispute` - Start a global dispute.
- `destroy_global_dispute` - Allow the users to get their voting funds and fee
  payments back.
