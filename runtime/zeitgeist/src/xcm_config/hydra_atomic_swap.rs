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

pub const HYDRA_PARA_ID: u32 = 2034;

pub struct AllowHydraDxAtomicSwap;

impl Contains<(MultiLocation, Xcm<RuntimeCall>)> for AllowHydraDxAtomicSwap {
    fn contains((ref origin, ref msg): &(MultiLocation, Xcm<RuntimeCall>)) -> bool {
        match origin {
            MultiLocation { parents: 0, interior: Junctions::X1(AccountId32 { .. }) } => {
                // TODO if msg matches HydraDX atomic swap messages then true, otherwise false
                // TODO figure out which stablecoin pairs to swap atomically with HydraDX
                let mut it = msg.inner().iter();
                match (it.next(), it.next(), it.next()) {
                    (
                        Some(SetFeesMode { jit_withdraw: true }),
                        Some(TransferReserveAsset { assets, dest, xcm }),
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
                        let valid_dest = *dest == MultiLocation::new(1, X1(Parachain(HYDRA_PARA_ID)));
                        // TODO Do we want to check the assets from TransferReserveAsset?
                    }
                    _ => return false,
                }

                false
            }
            _ => false,
        }
    }
}
