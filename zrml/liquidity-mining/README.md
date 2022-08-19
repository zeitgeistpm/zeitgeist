# Liquidity Mining Module

## Overview

Manages and distributes incentives to liquidity providers

Each block has a maximum allowed amount of ZTG that is distributed among the `PoolShare`
owners of that same block. Over time this amount will increase until a market closes and
then all rewards will be distributed accordingly.

This pallet is mostly self-contained and only need to know about the native currency. To
interact with its functionalities, please use the provided API.
