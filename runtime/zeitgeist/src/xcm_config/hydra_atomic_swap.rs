// Copyright 2024 Forecasting Technologies LTD.
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
#![cfg(feature = "parachain")]

use crate::{ParachainInfo, RuntimeCall};
use frame_support::{parameter_types, traits::Contains};
use xcm::prelude::*;

parameter_types! {
    pub const HydraDxParachainId: u32 = 2034;
    pub HydraDxMultiLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(HydraDxParachainId::get())));
}

pub struct AllowHydraDxAtomicSwap;

impl Contains<(MultiLocation, Xcm<RuntimeCall>)> for AllowHydraDxAtomicSwap {
    fn contains((ref origin, ref msg): &(MultiLocation, Xcm<RuntimeCall>)) -> bool {
        // allow root to execute XCM
        if origin == &MultiLocation::here() {
            return true;
        }

        match origin {
            // the outgoing XCMs to HydraDX
            MultiLocation { parents: 0, interior: Junctions::X1(AccountId32 { .. }) } => {
                match msg.inner() {
                    [
                        SetFeesMode { jit_withdraw: true },
                        TransferReserveAsset { assets: _, dest, xcm },
                    ] => {
                        let valid_xcm = matches!(
                            xcm.inner(),
                            [BuyExecution { .. }, ExchangeAsset { .. }, DepositAsset { .. }]
                        );
                        let valid_dest = *dest == HydraDxMultiLocation::get();
                        // TODO Do we want to check the multi assets and fees from TransferReserveAsset and ExchangeAsset and BuyExecution and DepositAsset? If yes, we could check this here too!
                        // TODO I suggest a pattern matching to only allow our stablecoin assets here. On the other hand this would require us to change this pattern if we change the configuration of stablecoins we have
                        valid_xcm && valid_dest
                    }
                    [WithdrawAsset(_), InitiateReserveWithdraw { assets: _, reserve: _, xcm }] => {
                        let xcm_3 = match xcm.inner() {
                            [BuyExecution { .. }, DepositReserveAsset { assets: _, dest, xcm }]
                                if *dest == HydraDxMultiLocation::get() =>
                            {
                                xcm
                            }
                            _ => return false,
                        };
                        let xcm_2 = match xcm_3.inner() {
                            [
                                BuyExecution { .. },
                                ExchangeAsset { .. },
                                InitiateReserveWithdraw { assets: _, reserve: _, xcm },
                            ] => xcm,
                            _ => return false,
                        };
                        let xcm_1 = match xcm_2.inner() {
                            // TODO maybe dest verification?
                            [
                                BuyExecution { .. },
                                DepositReserveAsset { assets: _, dest: _, xcm },
                            ] => xcm,
                            _ => return false,
                        };

                        return matches!(xcm_1.inner(), [BuyExecution { .. }, DepositAsset { .. }]);
                    }
                    _ => false,
                }
            }
            // the incoming XCMs from HydraDX
            MultiLocation { parents: 1, interior: Junctions::X1(Parachain(para_id)) } => {
                if *para_id != u32::from(ParachainInfo::parachain_id()) {
                    return false;
                }

                match msg.inner() {
                    // TODO maybe check the assets here?
                    // TODO or is it parents: 1 ?
                    // TODO or is it interior: Junctions::X2(Parachain(zeitgeist_parachain_id), AccountId32 { .. }) ?
                    [
                        BuyExecution { .. },
                        DepositAsset {
                            assets: _,
                            beneficiary:
                                MultiLocation {
                                    parents: 0,
                                    interior: Junctions::X1(AccountId32 { .. }),
                                },
                        },
                    ] => true,
                    _ => false,
                }
            }
            _ => false,
        }
    }
}
