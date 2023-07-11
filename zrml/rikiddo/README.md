# Rikiddo Module

Generic and modular implemenation of Rikiddo market scoring rule.

## Overview

Provides traits and implementations for sigmoid fee caluclation, calculation of
ema based on market volume, LMSR and Rikiddo using sigmoid fee calculation and
two ema periods.

Rikiddo is a liquidity-sensitive logarithm market scoring algorithm, which can
be used to determine the prices of event assets and their corresponding
probabilities. It incorporates historical trading data to optimize it's
reactiveness to abrupt and longer lasting changes in the market trend. More
information at [blog.zeitgeist.pm].

[blog.zeitgeist.pm]:
  https://blog.zeitgeist.pm/introducing-zeitgeists-rikiddo-scoring-rule/
