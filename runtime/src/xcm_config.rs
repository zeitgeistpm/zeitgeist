use crate::{
    AccountId, Ancestry, Balance, Balances, Call, Currency, MaxInstructions, Origin,
    ParachainSystem, PolkadotXcm, RelayChainOrigin, RelayLocation, RelayNetwork, Runtime,
    UnitWeightCost, UnknownTokens, XcmpQueue, ZeitgeistTreasuryAccount,
};
use frame_support::{match_types, parameter_types, traits::Everything, weights::IdentityFee};
use orml_xcm_support::{DepositToAlternative, IsNativeConcrete, MultiCurrencyAdapter};
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use sp_runtime::traits::Convert;
use xcm::latest::{
    prelude::{Concrete, GeneralKey, MultiAsset, Parachain, X1, X2},
    BodyId, Junction, Junctions, MultiLocation,
};
use xcm_builder::{
    AccountId32Aliases, AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom,
    FixedWeightBounds, LocationInverter, NativeAsset, ParentIsPreset, RelayChainAsNative,
    SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
    SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit, UsingComponents,
};
use xcm_executor::Config;
use zeitgeist_primitives::types::Asset;

pub struct XcmConfig;

/// The main XCM config
/// This is where we configure the core of our XCM integrations: how tokens are transferred,
/// how fees are calculated, what barriers we impose on incoming XCM messages, etc.
impl Config for XcmConfig {
    /// The handler for when there is an instruction to claim assets.
    type AssetClaims = PolkadotXcm;
    /// How to withdraw and deposit an asset.
    type AssetTransactor = MultiAssetTransactor;
    /// The general asset trap - handler for when assets are left in the Holding Register at the
    /// end of execution.
    type AssetTrap = PolkadotXcm;
    /// TODO: Additional filters that specify whether the XCM call should be executed at all.
    type Barrier = Barrier;
    /// The outer call dispatch type.
    type Call = Call;
    /// TODO: Combinations of (Location, Asset) pairs which are trusted as reserves
    type IsReserve = NativeAsset;
    /// Combinations of (Location, Asset) pairs which we trust as teleporters.
    type IsTeleporter = ();
    /// Means of inverting a location.
    type LocationInverter = LocationInverter<Ancestry>;
    /// TODO: How to get a call origin from a `OriginKind` value.
    type OriginConverter = XcmOriginToTransactDispatchOrigin;
    /// Module that handles responses of queries.
    type ResponseHandler = PolkadotXcm;
    /// Module that handles subscription requests.
    type SubscriptionService = PolkadotXcm;
    /// TODO: The means of purchasing weight credit for XCM execution.
    type Trader = UsingComponents<IdentityFee<Balance>, RelayLocation, AccountId, Balances, ()>;
    /// TODO: The means of determining an XCM message's weight.
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    /// How to send an onward XCM message.
    type XcmSender = XcmRouter;
}

pub type Barrier = (
    TakeWeightCredit,
    AllowTopLevelPaidExecutionFrom<Everything>,
    AllowUnpaidExecutionFrom<ParentOrParentsUnitPlurality>,
    // ^^^ Parent and its exec plurality get free execution
);
type AssetT = <Runtime as orml_tokens::Config>::CurrencyId;

parameter_types! {
    pub CheckAccount: AccountId = PolkadotXcm::check_account();
}

/// Means for transacting assets on this chain.
pub type MultiAssetTransactor = MultiCurrencyAdapter<
    // All known Assets will be processed by the following MultiCurrency implementation.
    Currency,
    // Any unknown Assets will be processed by the following implementation.
    UnknownTokens,
    // This means that this adapter should handle any token that `AssetConvert` can convert
    // using Currency and UnknownTokens in all other cases.
    IsNativeConcrete<AssetT, AssetConvert>,
    // Our chain's account ID type (we can't get away without mentioning it explicitly).
    AccountId,
    // Convert an XCM `MultiLocation` into a local account id.
    LocationToAccountId,
    // The AssetId that corresponds to the native currency.
    AssetT,
    // Struct that provides functions to convert `Asset` <=> `MultiLocation`.
    AssetConvert,
    // In case of deposit failure, known assets will be placed in treasury.
    DepositToAlternative<ZeitgeistTreasuryAccount, Currency, AssetT, AccountId, Balance>,
>;

/// Listing of parachains we integrate with.
/// For each parachain, we are interested in stating their parachain ID
/// as well as any of their token key ID that we possibly support in our
/// XCM configuration. These token keys are defined in said parachain
/// and must always match the value there defined, which is expected to
/// never change once defined since they help define the canonical id
/// of said tokens in the network, which is relevant for XCM transfers.
pub mod parachains {
    pub mod karura {
        pub const ID: u32 = 2000;
        pub const AUSD_KEY: &[u8] = &[0, 129];
    }

    pub mod zeitgeist {
        pub const ID: u32 = 2101;
        pub const ZTG_KEY: &[u8] = &[0, 1];
    }
}

/// AssetConvert
/// This type implements conversions from our `Asset` type into `MultiLocation` and vice-versa.
/// A currency locally is identified with a `Asset` variant but in the network it is identified
/// in the form of a `MultiLocation`, in this case a pair (Para-Id, Currency-Id).
pub struct AssetConvert;
// TODO: Use KSM or ROC depending on runtime that's built
const RELAY_CURRENCY: AssetT = Asset::Roc;

/// Convert our `Asset` type into its `MultiLocation` representation.
/// Other chains need to know how this conversion takes place in order to
/// handle it on their side.
impl Convert<AssetT, Option<MultiLocation>> for AssetConvert {
    fn convert(id: AssetT) -> Option<MultiLocation> {
        let x = match id {
            RELAY_CURRENCY => MultiLocation::parent(),
            Asset::Ausd => MultiLocation::new(
                1,
                X2(
                    Parachain(parachains::karura::ID),
                    GeneralKey(parachains::karura::AUSD_KEY.into()),
                ),
            ),
            Asset::Ztg => MultiLocation::new(
                1,
                X2(
                    Parachain(parachains::zeitgeist::ID),
                    GeneralKey(parachains::zeitgeist::ZTG_KEY.to_vec()),
                ),
            ),
            _ => return None,
        };
        Some(x)
    }
}

/// Convert an incoming `MultiLocation` into a `Asset` if possible.
/// Here we need to know the canonical representation of all the tokens we handle in order to
/// correctly convert their `MultiLocation` representation into our internal `Asset` type.
impl xcm_executor::traits::Convert<MultiLocation, AssetT> for AssetConvert {
    fn convert(location: MultiLocation) -> Result<AssetT, MultiLocation> {
        if location == MultiLocation::parent() {
            return Ok(RELAY_CURRENCY);
        }

        match location.clone() {
            MultiLocation { parents: 0, interior: X1(GeneralKey(key)) } => match &key[..] {
                parachains::zeitgeist::ZTG_KEY => Ok(Asset::Ztg),
                _ => Err(location.clone()),
            },
            MultiLocation { parents: 1, interior: X2(Parachain(para_id), GeneralKey(key)) } => {
                match para_id {
                    parachains::zeitgeist::ID => match &key[..] {
                        parachains::zeitgeist::ZTG_KEY => Ok(Asset::Ztg),
                        _ => Err(location.clone()),
                    },

                    parachains::karura::ID => match &key[..] {
                        parachains::karura::AUSD_KEY => Ok(Asset::Ausd),
                        _ => Err(location.clone()),
                    },
                    _ => Err(location.clone()),
                }
            }
            _ => Err(location.clone()),
        }
    }
}

impl Convert<MultiAsset, Option<AssetT>> for AssetConvert {
    fn convert(asset: MultiAsset) -> Option<AssetT> {
        if let MultiAsset { id: Concrete(location), .. } = asset {
            <AssetConvert as xcm_executor::traits::Convert<_, _>>::convert(location).ok()
        } else {
            None
        }
    }
}

impl Convert<MultiLocation, Option<AssetT>> for AssetConvert {
    fn convert(location: MultiLocation) -> Option<AssetT> {
        <AssetConvert as xcm_executor::traits::Convert<_, _>>::convert(location).ok()
    }
}

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
