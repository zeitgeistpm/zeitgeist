use crate::{
    AccountId, Ancestry, Balance, Balances, Barrier, Call, LocalAssetTransactor, RocLocation,
    UnitWeightCost, XcmOriginToTransactDispatchOrigin, XcmRouter,
};
use frame_support::weights::IdentityFee;
use xcm_builder::{FixedWeightBounds, LocationInverter, NativeAsset, UsingComponents};
use xcm_executor::Config;

pub struct XcmConfig;

impl Config for XcmConfig {
    type AssetTransactor = LocalAssetTransactor;
    type Barrier = Barrier;
    type Call = Call;
    type IsReserve = NativeAsset;
    type IsTeleporter = NativeAsset;
    type LocationInverter = LocationInverter<Ancestry>;
    type OriginConverter = XcmOriginToTransactDispatchOrigin;
    type ResponseHandler = ();
    type Trader = UsingComponents<IdentityFee<Balance>, RocLocation, AccountId, Balances, ()>;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call>;
    type XcmSender = XcmRouter;
}
