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

use crate::RuntimeCall;
use frame_support::{parameter_types, traits::Contains};
use xcm::prelude::*;

parameter_types! {
    pub const HydraDxParachainId: u32 = 2034;
    pub HydraDxMultiLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(HydraDxParachainId::get())));
}

pub struct AllowHydraDxAtomicSwap;

impl Contains<(MultiLocation, Xcm<RuntimeCall>)> for AllowHydraDxAtomicSwap {
    fn contains((ref origin, ref msg): &(MultiLocation, Xcm<RuntimeCall>)) -> bool {
        // TODO incoming xcm from HyrdaDX should be allowed here
        match origin {
            MultiLocation { parents: 0, interior: Junctions::X1(AccountId32 { .. }) } => {
                // TODO figure out which stablecoin pairs to swap atomically with HydraDX
                match msg.inner() {
                    [
                        SetFeesMode { jit_withdraw: true },
                        TransferReserveAsset { assets: _, dest, xcm },
                    ] => {
                        let valid_xcm = match xcm.inner() {
                            [BuyExecution { .. }, ExchangeAsset { .. }, DepositAsset { .. }] => {
                                true
                            }
                            _ => false,
                        };
                        let valid_dest = *dest == HydraDxMultiLocation::get();
                        // TODO Do we want to check the assets from TransferReserveAsset? If yes, we could check this here too!
                        return valid_xcm && valid_dest;
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
                        return match xcm_1.inner() {
                            [BuyExecution { .. }, DepositAsset { .. }] => true,
                            _ => false,
                        };
                    }
                    _ => return false,
                }
            }
            _ => false,
        }
    }
}
