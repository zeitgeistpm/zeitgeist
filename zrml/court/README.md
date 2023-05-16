# Court

A pallet for stake-weighted plurality decision making.

- [`Call`]()
- [`Config`]()
- [`Error`]()
- [`Event`]()

## Overview

Court is a market dispute resolution mechanism. It is responsible for ensuring 
that the truth is added to the blockchain. If someone provides false
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
