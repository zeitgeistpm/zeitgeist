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
use frame_support::traits::Contains;
use xcm::prelude::*;

pub const HYDRA_DX_PARA_ID: u32 = 2034;

pub struct AllowHydraDxAtomicSwap;

impl Contains<(MultiLocation, Xcm<RuntimeCall>)> for AllowHydraDxAtomicSwap {
    fn contains((ref origin, ref msg): &(MultiLocation, Xcm<RuntimeCall>)) -> bool {
        // TODO outsource this into parameter_types!
        let hydra_dx_loc: MultiLocation = MultiLocation::new(1, X1(Parachain(HYDRA_DX_PARA_ID)));
        let matches_inner_0 = |xcm: Xcm<()>| -> bool {
            let mut xit = xcm.inner().iter();
            match (xit.next(), xit.next(), xit.next()) {
                (Some(BuyExecution { .. }), Some(DepositAsset { .. }), None) => true,
                _ => false,
            }
        };
        let matches_inner_1 = |xcm: Xcm<()>| -> Option<Xcm<()>> {
            let mut xit = xcm.inner().iter();
            match (xit.next(), xit.next(), xit.next()) {
                (
                    Some(BuyExecution { .. }),
                    // TODO maybe dest verification?
                    Some(DepositReserveAsset { assets: _, dest: _, xcm }),
                    None,
                ) => Some(xcm.clone()),
                _ => None,
            }
        };
        let matches_inner_2 = |xcm: Xcm<()>| -> Option<Xcm<()>> {
            let mut xit = xcm.inner().iter();
            match (xit.next(), xit.next(), xit.next(), xit.next()) {
                (
                    Some(BuyExecution { .. }),
                    Some(ExchangeAsset { .. }),
                    Some(InitiateReserveWithdraw { assets: _, reserve: _, xcm }),
                    None,
                ) => Some(xcm.clone()),
                _ => None,
            }
        };
        let matches_inner_3 = |xcm: &Xcm<()>| -> Option<Xcm<()>> {
            let mut xit = xcm.inner().iter();
            match (xit.next(), xit.next(), xit.next()) {
                (
                    Some(BuyExecution { .. }),
                    Some(DepositReserveAsset { assets: _, dest, xcm }),
                    None,
                ) if *dest == hydra_dx_loc => Some(xcm.clone()),
                _ => None,
            }
        };
        match origin {
            MultiLocation { parents: 0, interior: Junctions::X1(AccountId32 { .. }) } => {
                // TODO figure out which stablecoin pairs to swap atomically with HydraDX
                let mut it = msg.inner().iter();
                match (it.next(), it.next(), it.next()) {
                    (
                        Some(SetFeesMode { jit_withdraw: true }),
                        Some(TransferReserveAsset { assets: _, dest, xcm }),
                        None,
                    ) => {
                        let mut xit = xcm.inner().iter();
                        let valid_xcm = match (xit.next(), xit.next(), xit.next(), xit.next()) {
                            (
                                Some(BuyExecution { .. }),
                                Some(ExchangeAsset { .. }),
                                Some(DepositAsset { .. }),
                                None,
                            ) => true,
                            _ => false,
                        };
                        let valid_dest = *dest == hydra_dx_loc;
                        // TODO Do we want to check the assets from TransferReserveAsset? If yes, we could check this here too!
                        return valid_xcm && valid_dest;
                    }
                    (
                        Some(WithdrawAsset(_)),
                        Some(InitiateReserveWithdraw { assets: _, reserve: _, xcm }),
                        None,
                    ) => {
                        let xcm_3_opt = matches_inner_3(xcm);
                        let xcm_3 = match xcm_3_opt {
                            None => return false,
                            Some(xcm_3) => xcm_3,
                        };
                        let xcm_2_opt = matches_inner_2(xcm_3);
                        let xcm_2 = match xcm_2_opt {
                            None => return false,
                            Some(xcm_2) => xcm_2,
                        };
                        let xcm_1_opt = matches_inner_1(xcm_2);
                        let xcm_1 = match xcm_1_opt {
                            None => return false,
                            Some(xcm_1) => xcm_1,
                        };
                        return matches_inner_0(xcm_1);
                    }
                    _ => return false,
                }
            }
            _ => false,
        }
    }
}
