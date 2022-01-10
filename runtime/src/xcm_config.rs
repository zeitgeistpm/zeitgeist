use crate::{
    AccountId, Ancestry, Balance, Balances, Barrier, Call, LocalAssetTransactor, PolkadotXcm,
    RelayChainLocation, UnitWeightCost, XcmOriginToTransactDispatchOrigin, XcmRouter,
};
use frame_support::weights::IdentityFee;
use xcm_builder::{FixedWeightBounds, LocationInverter, NativeAsset, UsingComponents};
use xcm_executor::Config;
use zeitgeist_primitives::constants::MaxInstructions;

pub struct XcmConfig;

impl Config for XcmConfig {
    type AssetTransactor = LocalAssetTransactor;
    type Barrier = Barrier;
    type Call = Call;
    type IsReserve = NativeAsset;
    type IsTeleporter = ();
    type LocationInverter = LocationInverter<Ancestry>;
    type OriginConverter = XcmOriginToTransactDispatchOrigin;
    type ResponseHandler = ();
    type SubscriptionService = PolkadotXcm;
    type Trader =
        UsingComponents<IdentityFee<Balance>, RelayChainLocation, AccountId, Balances, ()>;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type XcmSender = XcmRouter;
}
