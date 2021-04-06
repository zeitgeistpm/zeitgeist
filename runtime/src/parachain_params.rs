use crate::{AccountId, Balances, Origin, ParachainInfo};
use frame_support::parameter_types;
use polkadot_parachain::primitives::Sibling;
use xcm::v0::{Junction, MultiLocation, NetworkId};
use xcm_builder::{
    AccountId32Aliases, CurrencyAdapter, ParentIsDefault, RelayChainAsNative,
    SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
    SovereignSignedViaLocation,
};
use xcm_executor::traits::IsConcrete;

parameter_types! {
    pub Ancestry: MultiLocation = Junction::Parachain { id: ParachainInfo::parachain_id().into() }.into();
    pub const RococoLocation: MultiLocation = MultiLocation::X1(Junction::Parent);
    pub const RococoNetwork: NetworkId = NetworkId::Polkadot;
    pub RelayChainOrigin: Origin = cumulus_pallet_xcm_handler::Origin::Relay.into();
}

pub type LocalAssetTransactor =
    CurrencyAdapter<Balances, IsConcrete<RococoLocation>, LocationConverter, AccountId>;
pub type LocalOriginConverter = (
    SovereignSignedViaLocation<LocationConverter, Origin>,
    RelayChainAsNative<RelayChainOrigin, Origin>,
    SiblingParachainAsNative<cumulus_pallet_xcm_handler::Origin, Origin>,
    SignedAccountId32AsNative<RococoNetwork, Origin>,
);
pub type LocationConverter = (
    ParentIsDefault<AccountId>,
    SiblingParachainConvertsVia<Sibling, AccountId>,
    AccountId32Aliases<RococoNetwork, AccountId>,
);
