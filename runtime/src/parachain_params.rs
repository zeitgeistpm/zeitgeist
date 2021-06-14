use crate::{
    AccountId, Balances, Origin, ParachainInfo, ParachainSystem, XcmpQueue, MAXIMUM_BLOCK_WEIGHT,
};
use frame_support::{
    parameter_types,
    traits::{All, IsInVec},
    weights::Weight,
};
use polkadot_parachain::primitives::Sibling;
use sp_std::{vec, vec::Vec};
use xcm::v0::{Junction, MultiLocation, NetworkId};
use xcm_builder::{
    AccountId32Aliases, AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, CurrencyAdapter,
    IsConcrete, ParentAsSuperuser, ParentIsDefault, RelayChainAsNative, SiblingParachainAsNative,
    SiblingParachainConvertsVia, SignedAccountId32AsNative, SovereignSignedViaLocation,
    TakeWeightCredit,
};

parameter_types! {
    pub AllowUnpaidFrom: Vec<MultiLocation> = vec![ MultiLocation::X1(Junction::Parent) ];
    pub Ancestry: MultiLocation = MultiLocation::X1(Junction::Parachain(ParachainInfo::parachain_id().into()));
    pub const MaxDownwardMessageWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 10;
    pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
    pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
    pub const RocLocation: MultiLocation = MultiLocation::X1(Junction::Parent);
    pub const RococoLocation: MultiLocation = MultiLocation::X1(Junction::Parent);
    pub const RococoNetwork: NetworkId = NetworkId::Polkadot;
    pub const WeightPrice: (MultiLocation, u128) = (MultiLocation::X1(Junction::Parent), 1_000);
    pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
    pub UnitWeightCost: Weight = 1_000;
}

pub type Barrier = (
    TakeWeightCredit,
    AllowTopLevelPaidExecutionFrom<All<MultiLocation>>,
    AllowUnpaidExecutionFrom<IsInVec<AllowUnpaidFrom>>,
);
pub type LocalAssetTransactor =
    CurrencyAdapter<Balances, IsConcrete<RocLocation>, LocationToAccountId, AccountId, ()>;
pub type LocalOriginToLocation = ();
pub type LocationToAccountId = (
    ParentIsDefault<AccountId>,
    SiblingParachainConvertsVia<Sibling, AccountId>,
    AccountId32Aliases<RococoNetwork, AccountId>,
);
pub type XcmOriginToTransactDispatchOrigin = (
    SovereignSignedViaLocation<LocationToAccountId, Origin>,
    RelayChainAsNative<RelayChainOrigin, Origin>,
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
    ParentAsSuperuser<Origin>,
    SignedAccountId32AsNative<RococoNetwork, Origin>,
);
pub type XcmRouter = (cumulus_primitives_utility::ParentAsUmp<ParachainSystem>, XcmpQueue);
