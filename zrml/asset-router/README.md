# Asset Router

The asset router allows to interact with different asset classes using one
overaching asset class. The caller is not required to be aware of which pallet
handles the asset class of the asset in question, as the asset router internally
routes the call to the appropriate pallet as defined in the pallet's
configuration. It implements various ORML `MultiCurrency` traits as well as
various `Fungible` traits, thus it can be used in other pallets that require
those implementation (such as ORML Currencies). The asset router also provides a
garbage collector for destructible assets, that handles asset destruction
whenever on-chain execution time is available.
