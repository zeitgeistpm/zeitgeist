// Copyright 2021-2022 Zeitgeist PM LLC.
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

use crate::{
    AccountId, Ancestry, Balance, Balances, Barrier, Call, LocalAssetTransactor, MaxInstructions,
    PolkadotXcm, RelayLocation, UnitWeightCost, XcmOriginToTransactDispatchOrigin, XcmRouter,
};
use frame_support::weights::IdentityFee;
use xcm_builder::{FixedWeightBounds, LocationInverter, NativeAsset, UsingComponents};
use xcm_executor::Config;

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
