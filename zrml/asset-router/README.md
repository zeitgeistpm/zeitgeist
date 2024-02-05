# Asset Router

The asset router allows to interact with different asset classes using one
overaching asset class. The caller is not required to be aware of which pallet
handles the asset class of the asset in question, as the asset router internally
routes the call to the appropriate pallet as defined in the pallet's
configuration. It implements various ORML `MultiCurrency` traits as well as
various `Fungible` traits, thus it can be used in other pallets that require
those implementation (such as ORML Currencies). The asset router also provides
managed asset destruction, that handles asset destruction for all the assets
registered through the `ManagedDestroy` interface whenever on-chain execution
time is available.

## Managed Asset Destruction

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
