# Court

A pallet for stake-weighted plurality decision making.

- [`Call`]()
- [`Config`]()
- [`Error`]()
- [`Event`]()

## Overview

The court system is responsible for ensuring that the truth is added to the
blockchain. Prediction markets, which depend on accurate information, reward
traders who base their decisions on truthful data. If someone provides false
information, they will be punished, while those who share accurate information
will be rewarded.

## Terminology

- **Aggregation Period:** The period in which the actively participating jurors
  need to reveal their vote secrets.

## Interface

### Dispatches

#### Public Dispatches

- `join_court` - Join the court with a stake to become a juror in order to get
  the stake-weighted chance to be selected for decision making.
- `delegate` - Join the court with a stake to become a delegator in order to
  delegate the voting power to actively participating jurors.
- `prepare_exit_court` - Prepare as a court participant to leave the court
  system.
- `exit_court` - Exit the court system in order to get the stake back.
- `vote` - An actively participating juror votes secretely on a specific court
  case, in which the juror got selected.
- `denounce_vote` - Denounce a selected and active juror, if the secret and vote
  is known before the actual reveal period.
- `reveal_vote` - An actively participating juror reveals the previously casted
  secret vote.
- `appeal` - After the reveal phase (aggregation period), the jurors decision
  can be appealed.
- `reassign_juror_stakes` - After the appeal period is over, losers pay the
  winners for the jurors and delegators.

#### `MonetaryGovernanceOrigin` Dispatches

- `set_inflation` - Set the yearly inflation rate of the court system.

#### Private Pallet API

- `on_dispute` - Initialise a new court case.
- `on_resolution` - Resolve an existing court case and return the decision
  winner.
- `exchange` - Slash the unjustified appealers, put those funds to the treasury
  and unreserve the justified appeal bonds.
- `get_auto_resolve` - Get the block number when `on_resolution` is evoked for a
  court case.
- `has_failed` - Determine whether the court mechanism failed to meet the
  preconditions to start a court case.
- `on_global_dispute` - Prepare the global dispute if the court mechanism has
  failed.
- `clear` - Clean up the storage items of a specific court case.
