# Styx Module

A module for burning native chain tokens in order to gain entry into a registry for off-chain use.

## Overview

The pallet lets the signer burn native tokens, and lets governance update the price.
In the zeitgeist ecosystem this grants the ability to claim the avatar of the signer.

## Interface

### Dispatches

#### Public Dispatches

- `cross` - Burns native chain tokens to cross, granting the ability to claim your zeitgeist avatar.

#### Admin Dispatches

The administrative dispatches are used to perform admin functions on chain:

- `set_burn_amount` - Sets the new burn price for the cross. Intended to be called by governance.

The origins from which the admin functions are called (`SetBurnAmountOrigin`) are mainly minimum vote proportions from council.
