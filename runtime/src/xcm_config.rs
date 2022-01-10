use crate::{
    AccountId, Ancestry, Balance, Balances, Barrier, Call, LocalAssetTransactor, MaxInstructions,
    PolkadotXcm, RelayChainLocation, UnitWeightCost, XcmOriginToTransactDispatchOrigin, XcmRouter,
};
use frame_support::weights::IdentityFee;
use xcm_builder::{FixedWeightBounds, LocationInverter, NativeAsset, UsingComponents};
use xcm_executor::Config;

/*

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
    type Call = Call;
    type XcmSender = XcmRouter;
    // How to withdraw and deposit an asset.
    type AssetTransactor = LocalAssetTransactor;
    type OriginConverter = XcmOriginToCallOrigin;
    type IsReserve = MultiNativeAsset;
    // Teleporting is disabled.
    type IsTeleporter = ();
    type LocationInverter = LocationInverter<Ancestry>;
    type Barrier = Barrier;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type Trader = Trader;
    type ResponseHandler = PolkadotXcm;
    type AssetTrap = PolkadotXcm;
    type AssetClaims = PolkadotXcm;
    type SubscriptionService = PolkadotXcm;
}

*/

pub struct XcmConfig;

impl Config for XcmConfig {
    type AssetClaims = PolkadotXcm;
    type AssetTransactor = LocalAssetTransactor;
    type AssetTrap = PolkadotXcm;
    type Barrier = Barrier;
    type Call = Call;
    type IsReserve = NativeAsset;
    type IsTeleporter = ();
    type LocationInverter = LocationInverter<Ancestry>;
    type OriginConverter = XcmOriginToTransactDispatchOrigin;
    type ResponseHandler = PolkadotXcm;
    type SubscriptionService = PolkadotXcm;
    type Trader =
        UsingComponents<IdentityFee<Balance>, RelayChainLocation, AccountId, Balances, ()>;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type XcmSender = XcmRouter;
}
