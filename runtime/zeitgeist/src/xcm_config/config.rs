// Copyright 2022 Zeitgeist PM LLC.
//
// This file is part of Zeitgeist.
//
// Zeitgeist is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// Zeitgeist is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Zeitgeist. If not, see <https://www.gnu.org/licenses/>.

use super::fees::{native_per_second, FixedConversionRateProvider};
use crate::{
    AccountId, Ancestry, AssetManager, AssetRegistry, Balance, Call, CurrencyId, MaxInstructions,
    Origin, ParachainInfo, ParachainSystem, PolkadotXcm, RelayChainOrigin, RelayNetwork,
    UnitWeightCost, UnknownTokens, XcmpQueue, ZeitgeistTreasuryAccount,
};

use frame_support::{parameter_types, traits::Everything, WeakBoundedVec};
use orml_asset_registry::{AssetRegistryTrader, FixedRateAssetRegistryTrader};
use orml_traits::{location::AbsoluteReserveProvider, MultiCurrency};
use orml_xcm_support::{
    DepositToAlternative, IsNativeConcrete, MultiCurrencyAdapter, MultiNativeAsset,
};
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use sp_runtime::traits::Convert;
use xcm::{
    latest::{
        prelude::{AccountId32, AssetId, Concrete, GeneralKey, MultiAsset, NetworkId, X1, X2},
        Junction, MultiLocation,
    },
    opaque::latest::Fungibility::Fungible,
};
use xcm_builder::{
    AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
    AllowTopLevelPaidExecutionFrom, FixedRateOfFungible, FixedWeightBounds, LocationInverter,
    ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
    SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, TakeRevenue,
    TakeWeightCredit,
};
use xcm_executor::Config;
use zeitgeist_primitives::types::Asset;

pub mod zeitgeist {
    #[cfg(test)]
    pub const ID: u32 = 2092;
    pub const KEY: &[u8] = &[0, 1];
}

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
    /// Additional filters that specify whether the XCM instruction should be executed at all.
    type Barrier = Barrier;
    /// The outer call dispatch type.
    type Call = Call;
    type CallDispatcher = Call;
    /// Combinations of (Location, Asset) pairs which are trusted as reserves.
    // Trust the parent chain, sibling parachains and children chains of this chain.
    type IsReserve = MultiNativeAsset<AbsoluteReserveProvider>;
    /// Combinations of (Location, Asset) pairs which we trust as teleporters.
    type IsTeleporter = ();
    /// Means of inverting a location.
    type LocationInverter = LocationInverter<Ancestry>;
    /// How to get a call origin from a `OriginKind` value.
    type OriginConverter = XcmOriginToTransactDispatchOrigin;
    /// Module that handles responses of queries.
    type ResponseHandler = PolkadotXcm;
    /// Module that handles subscription requests.
    type SubscriptionService = PolkadotXcm;
    /// The means of purchasing weight credit for XCM execution.
    type Trader = Trader;
    /// The means of determining an XCM message's weight.
    // Adds UnitWeightCost per instruction plus the weight of each instruction.
    // The total number of instructions are bounded by MaxInstructions
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    /// How to send an onward XCM message.
    type XcmSender = XcmRouter;
}

/// Additional filters that specify whether the XCM instruction should be executed at all.
pub type Barrier = (
    // Execution barrier that just takes max_weight from weight_credit
    TakeWeightCredit,
    // Ensures that execution time is bought with BuyExecution instruction
    AllowTopLevelPaidExecutionFrom<Everything>,
    // Expected responses are OK.
    AllowKnownQueryResponses<PolkadotXcm>,
    // Subscriptions for version tracking are OK.
    AllowSubscriptionsFrom<Everything>,
);

/// The means of purchasing weight credit for XCM execution.
/// Every token that is accepted for XC transfers should be handled here.
pub type Trader = (
    // In case the asset in question is the native currency, it will charge
    // the default base fee per second and deposits them into treasury
    FixedRateOfFungible<ZtgPerSecond, ToTreasury>,
    FixedRateOfFungible<ZtgPerSecondCanonical, ToTreasury>,
    // For all other assets the base fee per second will tried to be derived
    // through the `fee_factor` entry in the asset registry. If the asset is
    // not present in the asset registry, the default base fee per second is used.
    // Deposits all fees into the treasury.
    AssetRegistryTrader<
        FixedRateAssetRegistryTrader<FixedConversionRateProvider<AssetRegistry>>,
        ToTreasury,
    >,
);

pub struct ToTreasury;
impl TakeRevenue for ToTreasury {
    fn take_revenue(revenue: MultiAsset) {
        use xcm_executor::traits::Convert;

        if let MultiAsset { id: Concrete(location), fun: Fungible(amount) } = revenue {
            if let Ok(asset_id) =
                <AssetConvert as Convert<MultiLocation, CurrencyId>>::convert(location)
            {
                let _ = AssetManager::deposit(asset_id, &ZeitgeistTreasuryAccount::get(), amount);
            }
        }
    }
}

parameter_types! {
    pub CheckAccount: AccountId = PolkadotXcm::check_account();
    /// The amount of ZTG charged per second of execution (canonical multilocation).
    pub ZtgPerSecondCanonical: (AssetId, u128) = (
        MultiLocation::new(
            0,
            X1(general_key(zeitgeist::KEY)),
        ).into(),
        native_per_second(),
    );
    /// The amount of ZTG charged per second of execution.
    pub ZtgPerSecond: (AssetId, u128) = (
        MultiLocation::new(
            1,
            X2(Junction::Parachain(ParachainInfo::parachain_id().into()), general_key(zeitgeist::KEY)),
        ).into(),
        native_per_second(),
    );
}

/// Means for transacting assets on this chain.
pub type MultiAssetTransactor = MultiCurrencyAdapter<
    // All known Assets will be processed by the following MultiCurrency implementation.
    AssetManager,
    // Any unknown Assets will be processed by the following UnknownAsset implementation.
    UnknownTokens,
    // This means that this adapter should handle any token that `AssetConvert` can convert
    // using AssetManager and UnknownTokens in all other cases.
    IsNativeConcrete<CurrencyId, AssetConvert>,
    // Our chain's account ID type (we can't get away without mentioning it explicitly).
    AccountId,
    // Convert an XCM `MultiLocation` into a local account id.
    LocationToAccountId,
    // The AssetId that corresponds to the native currency.
    CurrencyId,
    // Struct that provides functions to convert `Asset` <=> `MultiLocation`.
    AssetConvert,
    // In case of deposit failure, known assets will be placed in treasury.
    DepositToAlternative<ZeitgeistTreasuryAccount, AssetManager, CurrencyId, AccountId, Balance>,
>;

/// AssetConvert
/// This type implements conversions from our `Asset` type into `MultiLocation` and vice-versa.
/// A currency locally is identified with a `Asset` variant but in the network it is identified
/// in the form of a `MultiLocation`, in this case a pair (Para-Id, Currency-Id).
pub struct AssetConvert;

/// Convert our `Asset` type into its `MultiLocation` representation.
/// Other chains need to know how this conversion takes place in order to
/// handle it on their side.
impl Convert<CurrencyId, Option<MultiLocation>> for AssetConvert {
    fn convert(id: CurrencyId) -> Option<MultiLocation> {
        match id {
            Asset::Ztg => Some(MultiLocation::new(
                1,
                X2(
                    Junction::Parachain(ParachainInfo::parachain_id().into()),
                    general_key(zeitgeist::KEY),
                ),
            )),
            Asset::ForeignAsset(_) => AssetRegistry::multilocation(&id).ok()?,
            _ => None,
        }
    }
}

/// Convert an incoming `MultiLocation` into a `Asset` if possible.
/// Here we need to know the canonical representation of all the tokens we handle in order to
/// correctly convert their `MultiLocation` representation into our internal `Asset` type.
impl xcm_executor::traits::Convert<MultiLocation, CurrencyId> for AssetConvert {
    fn convert(location: MultiLocation) -> Result<CurrencyId, MultiLocation> {
        match location.clone() {
            MultiLocation { parents: 0, interior: X1(GeneralKey(key)) } => {
                if &key[..] == zeitgeist::KEY {
                    return Ok(CurrencyId::Ztg);
                }

                Err(location)
            }
            MultiLocation {
                parents: 1,
                interior: X2(Junction::Parachain(para_id), GeneralKey(key)),
            } => {
                if para_id == u32::from(ParachainInfo::parachain_id()) {
                    if &key[..] == zeitgeist::KEY {
                        return Ok(CurrencyId::Ztg);
                    }

                    return Err(location);
                }

                AssetRegistry::location_to_asset_id(location.clone()).ok_or(location)
            }
            _ => AssetRegistry::location_to_asset_id(location.clone()).ok_or(location),
        }
    }
}

impl Convert<MultiAsset, Option<CurrencyId>> for AssetConvert {
    fn convert(asset: MultiAsset) -> Option<CurrencyId> {
        if let MultiAsset { id: Concrete(location), .. } = asset {
            <AssetConvert as xcm_executor::traits::Convert<_, _>>::convert(location).ok()
        } else {
            None
        }
    }
}

impl Convert<MultiLocation, Option<CurrencyId>> for AssetConvert {
    fn convert(location: MultiLocation) -> Option<CurrencyId> {
        <AssetConvert as xcm_executor::traits::Convert<_, _>>::convert(location).ok()
    }
}

pub struct AccountIdToMultiLocation;

impl Convert<AccountId, MultiLocation> for AccountIdToMultiLocation {
    fn convert(account: AccountId) -> MultiLocation {
        X1(AccountId32 { network: NetworkId::Any, id: account.into() }).into()
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
    // Native converter for Relay-chain (Parent) location; will convert to a `Relay` origin when
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

#[inline]
pub(crate) fn general_key(key: &[u8]) -> Junction {
    GeneralKey(WeakBoundedVec::force_from(key.to_vec(), None))
}
