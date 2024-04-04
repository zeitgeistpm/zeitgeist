// Copyright 2022-2024 Forecasting Technologies LTD.
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
    AccountId, AssetManager, AssetRegistry, Assets, Balance, MaxAssetsIntoHolding, MaxInstructions,
    ParachainInfo, ParachainSystem, PolkadotXcm, RelayChainOrigin, RelayNetwork, RuntimeCall,
    RuntimeOrigin, UnitWeightCost, UniversalLocation, UnknownTokens, XcmpQueue,
    ZeitgeistTreasuryAccount,
};
use alloc::vec::Vec;
use core::{cmp::min, marker::PhantomData};
use frame_support::{
    parameter_types,
    traits::{ConstU8, Everything, Get, Nothing},
};
use orml_asset_registry::{AssetRegistryTrader, FixedRateAssetRegistryTrader};
use orml_traits::{asset_registry::Inspect, location::AbsoluteReserveProvider};
use orml_xcm_support::{
    DepositToAlternative, IsNativeConcrete, MultiCurrencyAdapter, MultiNativeAsset,
};
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use sp_runtime::traits::Convert;
use xcm::{
    latest::{
        prelude::{AccountId32, AssetId, Concrete, GeneralKey, MultiAsset, XcmContext, X1, X2},
        Error as XcmError, Junction, MultiLocation, Result as XcmResult,
    },
    opaque::latest::Fungibility::Fungible,
};
use xcm_builder::{
    AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
    AllowTopLevelPaidExecutionFrom, FixedRateOfFungible, FixedWeightBounds, ParentIsPreset,
    RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
    SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, TakeRevenue,
    TakeWeightCredit,
};
use xcm_executor::{traits::TransactAsset, Assets as ExecutorAssets, Config};
use zeitgeist_primitives::{constants::BalanceFractionalDecimals, types::XcmAsset};

pub mod battery_station {
    #[cfg(test)]
    pub const ID: u32 = 2101;
    pub const KEY: &[u8] = &[0, 1];
}

pub struct XcmConfig;

/// The main XCM config
/// This is where we configure the core of our XCM integrations: how tokens are transferred,
/// how fees are calculated, what barriers we impose on incoming XCM messages, etc.
impl Config for XcmConfig {
    /// Handler for exchanging assets.
    type AssetExchanger = ();
    /// The handler for when there is an instruction to claim assets.
    type AssetClaims = PolkadotXcm;
    /// Handler for asset locking.
    type AssetLocker = ();
    /// How to withdraw and deposit an asset.
    type AssetTransactor = AlignedFractionalMultiAssetTransactor;
    /// The general asset trap - handler for when assets are left in the Holding Register at the
    /// end of execution.
    type AssetTrap = PolkadotXcm;
    /// Additional filters that specify whether the XCM instruction should be executed at all.
    type Barrier = Barrier;
    /// XCM will use this to dispatch any calls
    type CallDispatcher = RuntimeCall;
    /// Configure the fees.
    type FeeManager = ();
    /// Combinations of (Location, Asset) pairs which are trusted as reserves.
    // Trust the parent chain, sibling parachains and children chains of this chain.
    type IsReserve = MultiNativeAsset<AbsoluteReserveProvider>;
    /// Combinations of (Location, Asset) pairs which we trust as teleporters.
    type IsTeleporter = ();
    /// Maximum amount of tokens the holding register can store
    type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
    /// The method of exporting a message.
    type MessageExporter = ();
    /// How to get a call origin from a `OriginKind` value.
    type OriginConverter = XcmOriginToTransactDispatchOrigin;
    /// Information on all pallets.
    type PalletInstancesInfo = crate::AllPalletsWithSystem;
    /// Module that handles responses of queries.
    type ResponseHandler = PolkadotXcm;
    /// The outer call dispatch type.
    type RuntimeCall = RuntimeCall;
    /// The safe call filter for `Transact`.
    type SafeCallFilter = Nothing;
    /// Module that handles subscription requests.
    type SubscriptionService = PolkadotXcm;
    /// The means of purchasing weight credit for XCM execution.
    type Trader = Trader;
    /// The origin locations and specific universal junctions to which they are allowed to elevate
    /// themselves.
    type UniversalAliases = Nothing;
    /// This chain's Universal Location.
    type UniversalLocation = UniversalLocation;
    /// The means of determining an XCM message's weight.
    // Adds UnitWeightCost per instruction plus the weight of each instruction.
    // The total number of instructions are bounded by MaxInstructions
    type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
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
        use orml_traits::MultiCurrency;
        use xcm_executor::traits::Convert;

        if let MultiAsset { id: Concrete(location), fun: Fungible(_amount) } = revenue {
            if let Ok(asset_id) =
                <AssetConvert as Convert<MultiLocation, Assets>>::convert(location)
            {
                let adj_am =
                    AlignedFractionalMultiAssetTransactor::adjust_fractional_places(&revenue).fun;

                if let Fungible(amount) = adj_am {
                    let _ =
                        AssetManager::deposit(asset_id, &ZeitgeistTreasuryAccount::get(), amount);
                }
            }
        }
    }
}

parameter_types! {
    pub CheckAccount: AccountId = PolkadotXcm::check_account();
    /// The amount of ZTG charged per second of execution (canonical multilocation).
    pub ZtgPerSecondCanonical: (AssetId, u128, u128) = (
        MultiLocation::new(
            0,
            X1(general_key(battery_station::KEY)),
        ).into(),
        native_per_second(),
        0,
    );
    /// The amount of canonical ZTG charged per second of execution.
    pub ZtgPerSecond: (AssetId, u128, u128) = (
        MultiLocation::new(
            1,
            X2(Junction::Parachain(ParachainInfo::parachain_id().into()), general_key(battery_station::KEY)),
        ).into(),
        native_per_second(),
        0,
    );
}

/// A generic warpper around implementations of the (xcm-executor) `TransactAsset` trait.
///
/// Aligns the fractional decimal places of every incoming token with ZTG.
/// Reconstructs the original number of fractional decimal places of every outgoing token.
///
/// Important: Always use the global canonical representation of token balances in XCM.
/// Only during the interpretation of those XCM adjustments happens.
///
/// Important: The implementation does not support teleports.
#[allow(clippy::type_complexity)]
pub struct AlignedFractionalTransactAsset<
    AssetRegistry,
    CurrencyIdConvert,
    FracDecPlaces,
    TransactAssetDelegate,
> {
    _phantom: PhantomData<(AssetRegistry, CurrencyIdConvert, FracDecPlaces, TransactAssetDelegate)>,
}

impl<
    AssetRegistry: Inspect<AssetId = XcmAsset>,
    FracDecPlaces: Get<u8>,
    CurrencyIdConvert: Convert<MultiAsset, Option<Assets>>,
    TransactAssetDelegate: TransactAsset,
>
    AlignedFractionalTransactAsset<
        AssetRegistry,
        CurrencyIdConvert,
        FracDecPlaces,
        TransactAssetDelegate,
    >
{
    fn adjust_fractional_places(asset: &MultiAsset) -> MultiAsset {
        let (asset_id, amount) =
            if let Some(ref asset_id) = CurrencyIdConvert::convert(asset.clone()) {
                if let Fungible(amount) = asset.fun {
                    (*asset_id, amount)
                } else {
                    return asset.clone();
                }
            } else {
                return asset.clone();
            };

        let currency = if let Ok(currency) = XcmAsset::try_from(asset_id) {
            currency
        } else {
            return asset.clone();
        };

        let metadata = AssetRegistry::metadata(&currency);
        if let Some(metadata) = metadata {
            let mut asset_adjusted = asset.clone();
            let decimals = metadata.decimals;
            let native_decimals = u32::from(FracDecPlaces::get());

            asset_adjusted.fun = if decimals > native_decimals {
                let power = decimals.saturating_sub(native_decimals);
                let adjust_factor = 10u128.saturating_pow(power);
                // Floors the adjusted token amount, thus no tokens are generated
                Fungible(amount.saturating_div(adjust_factor))
            } else {
                let power = native_decimals.saturating_sub(decimals);
                let adjust_factor = 10u128.saturating_pow(power);
                Fungible(amount.saturating_mul(adjust_factor))
            };

            return asset_adjusted;
        }

        asset.clone()
    }
}

impl<
    AssetRegistry: Inspect<AssetId = XcmAsset>,
    CurrencyIdConvert: Convert<MultiAsset, Option<Assets>>,
    FracDecPlaces: Get<u8>,
    TransactAssetDelegate: TransactAsset,
> TransactAsset
    for AlignedFractionalTransactAsset<
        AssetRegistry,
        CurrencyIdConvert,
        FracDecPlaces,
        TransactAssetDelegate,
    >
{
    fn deposit_asset(
        asset: &MultiAsset,
        location: &MultiLocation,
        context: &XcmContext,
    ) -> XcmResult {
        let asset_adjusted = Self::adjust_fractional_places(asset);
        TransactAssetDelegate::deposit_asset(&asset_adjusted, location, context)
    }

    fn withdraw_asset(
        asset: &MultiAsset,
        location: &MultiLocation,
        maybe_context: Option<&XcmContext>,
    ) -> Result<ExecutorAssets, XcmError> {
        let asset_adjusted = Self::adjust_fractional_places(asset);
        TransactAssetDelegate::withdraw_asset(&asset_adjusted, location, maybe_context)
    }

    fn transfer_asset(
        asset: &MultiAsset,
        from: &MultiLocation,
        to: &MultiLocation,
        context: &XcmContext,
    ) -> Result<ExecutorAssets, XcmError> {
        let asset_adjusted = Self::adjust_fractional_places(asset);
        TransactAssetDelegate::transfer_asset(&asset_adjusted, from, to, context)
    }
}

pub type AlignedFractionalMultiAssetTransactor = AlignedFractionalTransactAsset<
    AssetRegistry,
    AssetConvert,
    ConstU8<{ BalanceFractionalDecimals::get() }>,
    MultiAssetTransactor,
>;

/// Means for transacting assets on this chain.
pub type MultiAssetTransactor = MultiCurrencyAdapter<
    // All known Assets will be processed by the following MultiCurrency implementation.
    AssetManager,
    // Any unknown Assets will be processed by the following UnknownAsset implementation.
    UnknownTokens,
    // This means that this adapter should handle any token that `AssetConvert` can convert
    // using AssetManager and UnknownTokens in all other cases.
    IsNativeConcrete<Assets, AssetConvert>,
    // Our chain's account ID type (we can't get away without mentioning it explicitly).
    AccountId,
    // Convert an XCM `MultiLocation` into a local account id.
    LocationToAccountId,
    // The AssetId that corresponds to the native currency.
    Assets,
    // Struct that provides functions to convert `Asset` <=> `MultiLocation`.
    AssetConvert,
    // In case of deposit failure, known assets will be placed in treasury.
    DepositToAlternative<ZeitgeistTreasuryAccount, AssetManager, Assets, AccountId, Balance>,
>;

/// AssetConvert
/// This type implements conversions from our `Asset` type into `MultiLocation` and vice-versa.
/// A currency locally is identified with a `Asset` variant but in the network it is identified
/// in the form of a `MultiLocation`, in this case a pair (Para-Id, Currency-Id).
pub struct AssetConvert;

/// Convert our `Asset` type into its `MultiLocation` representation.
/// Other chains need to know how this conversion takes place in order to
/// handle it on their side.
impl Convert<Assets, Option<MultiLocation>> for AssetConvert {
    fn convert(id: Assets) -> Option<MultiLocation> {
        match id {
            Assets::Ztg => Some(MultiLocation::new(
                1,
                X2(
                    Junction::Parachain(ParachainInfo::parachain_id().into()),
                    general_key(battery_station::KEY),
                ),
            )),
            Assets::ForeignAsset(_) => {
                let asset = XcmAsset::try_from(id).ok()?;
                AssetRegistry::multilocation(&asset).ok()?
            }
            _ => None,
        }
    }
}

impl Convert<XcmAsset, Option<MultiLocation>> for AssetConvert {
    fn convert(id: XcmAsset) -> Option<MultiLocation> {
        <Self as Convert<Assets, Option<MultiLocation>>>::convert(id.into())
    }
}

/// Convert an incoming `MultiLocation` into a `Asset` if possible.
/// Here we need to know the canonical representation of all the tokens we handle in order to
/// correctly convert their `MultiLocation` representation into our internal `Asset` type.
impl xcm_executor::traits::Convert<MultiLocation, Assets> for AssetConvert {
    fn convert(location: MultiLocation) -> Result<Assets, MultiLocation> {
        match location {
            MultiLocation { parents: 0, interior: X1(GeneralKey { data, length }) } => {
                let key = &data[..data.len().min(length as usize)];

                if key == battery_station::KEY {
                    return Ok(Assets::Ztg);
                }

                Err(location)
            }
            MultiLocation {
                parents: 1,
                interior: X2(Junction::Parachain(para_id), GeneralKey { data, length }),
            } => {
                let key = &data[..data.len().min(length as usize)];

                if para_id == u32::from(ParachainInfo::parachain_id()) {
                    if key == battery_station::KEY {
                        return Ok(Assets::Ztg);
                    }

                    return Err(location);
                }

                AssetRegistry::location_to_asset_id(location).ok_or(location).map(|a| a.into())
            }
            _ => AssetRegistry::location_to_asset_id(location).ok_or(location).map(|a| a.into()),
        }
    }
}

impl xcm_executor::traits::Convert<MultiLocation, XcmAsset> for AssetConvert {
    fn convert(location: MultiLocation) -> Result<XcmAsset, MultiLocation> {
        <Self as xcm_executor::traits::Convert<MultiLocation, Assets>>::convert(location)
            .and_then(|asset| asset.try_into().map_err(|_| location))
    }
}

impl Convert<MultiAsset, Option<Assets>> for AssetConvert {
    fn convert(asset: MultiAsset) -> Option<Assets> {
        if let MultiAsset { id: Concrete(location), .. } = asset {
            <AssetConvert as xcm_executor::traits::Convert<_, _>>::convert(location).ok()
        } else {
            None
        }
    }
}

impl Convert<MultiLocation, Option<Assets>> for AssetConvert {
    fn convert(location: MultiLocation) -> Option<Assets> {
        <AssetConvert as xcm_executor::traits::Convert<_, _>>::convert(location).ok()
    }
}

pub struct AccountIdToMultiLocation;

impl Convert<AccountId, MultiLocation> for AccountIdToMultiLocation {
    fn convert(account: AccountId) -> MultiLocation {
        X1(AccountId32 { network: None, id: account.into() }).into()
    }
}

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

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
    SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
    // Native converter for Relay-chain (Parent) location; will convert to a `Relay` origin when
    // recognized.
    RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
    // Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
    // recognized.
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
    // Native signed account converter; this just converts an `AccountId32` origin into a normal
    // `Origin::Signed` origin of the same 32-byte value.
    SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
    // Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
    XcmPassthrough<RuntimeOrigin>,
);

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
    // Two routers - use UMP to communicate with the relay chain:
    cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm, ()>,
    // ..and XCMP to communicate with the sibling chains.
    XcmpQueue,
);

/// Build a fixed-size array using as many elements from `src` as possible
/// without overflowing and ensuring that the array is 0 padded in the case
/// where `src.len()` is smaller than S.
fn vec_to_fixed_array<const S: usize>(src: Vec<u8>) -> [u8; S] {
    let mut dest = [0; S];
    let len = min(src.len(), S);
    dest[..len].copy_from_slice(&src.as_slice()[..len]);

    dest
}

/// A utils function to un-bloat and simplify the instantiation of
/// `GeneralKey` values
pub fn general_key(data: &[u8]) -> Junction {
    GeneralKey { length: data.len().min(32) as u8, data: vec_to_fixed_array(data.to_vec()) }
}
