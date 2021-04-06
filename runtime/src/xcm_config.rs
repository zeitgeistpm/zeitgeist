use crate::{Ancestry, Call, LocalAssetTransactor, LocalOriginConverter, XcmHandler};
use xcm_builder::LocationInverter;
use xcm_executor::{traits::NativeAsset, Config};

pub struct XcmConfig;

impl Config for XcmConfig {
    type AssetTransactor = LocalAssetTransactor;
    type Call = Call;
    type IsReserve = NativeAsset;
    type IsTeleporter = ();
    type LocationInverter = LocationInverter<Ancestry>;
    type OriginConverter = LocalOriginConverter;
    type XcmSender = XcmHandler;
}
