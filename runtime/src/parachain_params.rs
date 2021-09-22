use crate::{
    AccountId, Balances, Origin, ParachainInfo, ParachainSystem, XcmpQueue, MAXIMUM_BLOCK_WEIGHT,
};
use frame_support::{match_type, parameter_types, traits::Everything, weights::Weight};
use polkadot_parachain::primitives::Sibling;
use sp_runtime::SaturatedConversion;
use xcm::latest::{BodyId, Junction, Junctions, MultiLocation, NetworkId};
use xcm_builder::{
    AccountId32Aliases, AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, CurrencyAdapter,
    IsConcrete, ParentAsSuperuser, ParentIsDefault, RelayChainAsNative, SiblingParachainAsNative,
    SiblingParachainConvertsVia, SignedAccountId32AsNative, SovereignSignedViaLocation,
    TakeWeightCredit,
};
use zeitgeist_primitives::constants::MICRO;

match_type! {
    pub type ParentOrParentsUnitPlurality: impl Contains<MultiLocation> = {
        MultiLocation { parents: 1, interior: Junctions::Here } |
        MultiLocation { parents: 1, interior: Junctions::X1(Junction::Plurality { id: BodyId::Unit, .. }) }
    };
}
parameter_types! {
    pub Ancestry: MultiLocation = Junction::Parachain(ParachainInfo::parachain_id().into()).into();
    pub const RelayChainLocation: MultiLocation = MultiLocation::parent();
    pub const RelayChainNetwork: NetworkId = NetworkId::Kusama;
    pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
    pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
    pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
    pub UnitWeightCost: Weight = MICRO.saturated_into();
}

pub type Barrier = (
    TakeWeightCredit,
    AllowTopLevelPaidExecutionFrom<Everything>,
    AllowUnpaidExecutionFrom<ParentOrParentsUnitPlurality>,
);
pub type LocalAssetTransactor =
    CurrencyAdapter<Balances, IsConcrete<RelayChainLocation>, LocationToAccountId, AccountId, ()>;
pub type LocalOriginToLocation = ();
pub type LocationToAccountId = (
    ParentIsDefault<AccountId>,
    SiblingParachainConvertsVia<Sibling, AccountId>,
    AccountId32Aliases<RelayChainNetwork, AccountId>,
);
pub type XcmOriginToTransactDispatchOrigin = (
    SovereignSignedViaLocation<LocationToAccountId, Origin>,
    RelayChainAsNative<RelayChainOrigin, Origin>,
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
    ParentAsSuperuser<Origin>,
    SignedAccountId32AsNative<RelayChainNetwork, Origin>,
);
pub type XcmRouter = (cumulus_primitives_utility::ParentAsUmp<ParachainSystem, ()>, XcmpQueue);
