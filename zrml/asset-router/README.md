# Asset Router

The asset router allows to interact with different asset classes using one
overaching asset class. The caller is not required to be aware of which pallet
handles the asset class of the asset in question, as the asset router internally
routes the call to the appropriate pallet as defined in the pallet's
configuration.

## Overview

The asset router implements various ORML `MultiCurrency` traits as well as
various `Fungible` traits, thus it can be used in other pallets that require
those implementation (such as ORML Currencies). The asset router also provides
managed asset destruction, that handles asset destruction for all the assets
registered through the `ManagedDestroy` interface whenever on-chain execution
time is available. Finally, the asset router also provides a lazy migration
mechanism for asset types from the `CurrencyType` to the `MarketAssetType`.

### Terminology

- _Asset Type_: An asset type is used to represent assets that share the same
  purpose.
- _Asset Class_: An asset class is an overarching collection of multible asset
  types that share common properties.
- _Tokens_: Tokens are a countable number of instantiations of a specific asset
  type that can be moved between accounts.
- _Lazy Migration_: A lazy migration migrates data and the control over the data
  from a source to a destination over a prolonged amount of time, usually per
  request of the data or after expiry of the data.
- _Managed Asset Destruction_: A mechanism to automatically destroy an asset
  type.

### Managed Asset Destruction

Once an asset was registered for managed destruction, it's assigned a state and
stored in a sorted list within the `DestroyAssets` storage. Whenever weight is
available in a block, this pallet will process as many assets as possible from
that sorted list. To achieve that, it loops through all assets one by one and
for each asset, it runs through a state machine that ensures that every step
necessary to properly destroy an asset is executed and that the states are
updated accordingly. It might occur that the pallet that does the actual
destruction, i.e. that is invoked by the managed destruction routine to destroy
a specific asset (using the `Destroy` interface), throws an error. In that case
an asset is considered as `Indestructible` and stored in the
`IndestructibleAssets` storage, while also logging the incident.

### Lazy migration from `CurrencyClass` to `MarketAssetClass`

As some asset types within `CurrencyType` and `MarketAssetType` map to the same
asset type in the overarching `AssetType`, it is necessary to apply some
additional logic to determine when a function call with an asset of `AssetType`
should be invoked in `Currencies` and when it should be invoked in
`MarketAssets`. The approach this pallet uses is as follows:

- Try to convert `AssetType` into `MarketAssetType`
- On success, check if `MarketAssetType` exists.
  - If it does, invoke the function in `MarketAssets`
  - If it does not, try to convert to `CurrencyType`.
    - On success, invoke `Currencies`
    - On failure, invoke `MarketAssets`
- On failure, continue trying to convert into other known asset types.
