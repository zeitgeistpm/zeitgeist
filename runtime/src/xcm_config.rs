use crate::{
    AccountId, Ancestry, Balance, Balances, Call, MaxInstructions, Origin, ParachainSystem,
    PolkadotXcm, RelayChainOrigin, RelayLocation, RelayNetwork, UnitWeightCost, XcmpQueue, 
};
use frame_support::{match_types, traits::Everything, weights::IdentityFee};
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use xcm_builder::{
    AccountId32Aliases, AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, CurrencyAdapter,
    FixedWeightBounds, IsConcrete, LocationInverter, NativeAsset, ParentIsPreset,
    RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
    SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit,
    UsingComponents,
};
use xcm_executor::Config;
use xcm::latest::{BodyId, Junction, Junctions, MultiLocation};

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

pub type Barrier = (
    TakeWeightCredit,
    AllowTopLevelPaidExecutionFrom<Everything>,
    AllowUnpaidExecutionFrom<ParentOrParentsUnitPlurality>,
    // ^^^ Parent and its exec plurality get free execution
);

/// Means for transacting assets on this chain.
pub type LocalAssetTransactor = CurrencyAdapter<
    // Use this currency:
    Balances,
    // Use this currency when it is a fungible asset matching the given location or name:
    IsConcrete<RelayLocation>,
    // Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
    LocationToAccountId,
    // Our chain's account ID type (we can't get away without mentioning it explicitly):
    AccountId,
    // We don't track any teleports.
    (),
>;

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<Origin, AccountId, RelayNetwork>;

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
    // The parent (Relay-chain) origin converts to the parent `AccountId`.
    ParentIsPreset<AccountId>,
    // Sibling parachain origins convert to AccountId via the `ParaId::into`.
    SiblingParachainConvertsVia<Sibling, AccountId>,
    // Straight up local `AccountId32` origins just alias directly to `AccountId`.
    AccountId32Aliases<RelayNetwork, AccountId>,
);

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
    // Sovereign account converter; this attempts to derive an `AccountId` from the origin location
    // using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
    // foreign chains who want to have a local sovereign account on this chain which they control.
    SovereignSignedViaLocation<LocationToAccountId, Origin>,
    // Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
    // recognized.
    RelayChainAsNative<RelayChainOrigin, Origin>,
    // Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
    // recognized.
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
    // Native signed account converter; this just converts an `AccountId32` origin into a normal
    // `Origin::Signed` origin of the same 32-byte value.
    SignedAccountId32AsNative<RelayNetwork, Origin>,
    // Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
    XcmPassthrough<Origin>,
);

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
    // Two routers - use UMP to communicate with the relay chain:
    cumulus_primitives_utility::ParentAsUmp<ParachainSystem, ()>,
    // ..and XCMP to communicate with the sibling chains.
    XcmpQueue,
);

match_types! {
    pub type ParentOrParentsUnitPlurality: impl Contains<MultiLocation> = {
        MultiLocation { parents: 1, interior: Junctions::Here } |
        // Potentially change "Unit" to "Executive" for mainnet once we have separate runtimes
        MultiLocation { parents: 1, interior: Junctions::X1(Junction::Plurality { id: BodyId::Unit, .. }) }
    };
}
