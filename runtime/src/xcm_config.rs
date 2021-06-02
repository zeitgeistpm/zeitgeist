use crate::{
    Ancestry, Barrier, Call, LocalAssetTransactor, UnitWeightCost, WeightPrice,
    XcmOriginToTransactDispatchOrigin, XcmRouter,
};
use xcm_builder::{FixedRateOfConcreteFungible, FixedWeightBounds, LocationInverter, NativeAsset};
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
    type Trader = FixedRateOfConcreteFungible<WeightPrice>;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call>;
    type XcmSender = XcmRouter;
}
