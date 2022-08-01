use super::{
    AccountId, Ancestry, Balance, Balances, Barrier, Call, LocalAssetTransactor, MaxInstructions,
    PolkadotXcm, RelayLocation, UnitWeightCost, XcmOriginToTransactDispatchOrigin, XcmRouter,
};
use frame_support::weights::IdentityFee;
use xcm_builder::{FixedWeightBounds, LocationInverter, NativeAsset, UsingComponents};
use xcm_executor::Config;

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
    type Trader = UsingComponents<IdentityFee<Balance>, RelayLocation, AccountId, Balances, ()>;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type XcmSender = XcmRouter;
}
