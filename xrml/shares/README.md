# Shares

An implementation of the shares trait from the [`xrml-traits`](../traits) library.
The Shares pallet represents prediction market shares. Shares are identified by
a hash identifier. The hash is inherited from the pallet that uses the Shares
pallet, so special care should be paid from the developer that no collisions
occur.

