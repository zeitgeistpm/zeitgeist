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

cfg_if::cfg_if! {
    if #[cfg(feature = "parachain")] {
        pub mod cumulus_pallet_xcmp_queue;
        pub mod pallet_author_inherent;
        pub mod pallet_author_mapping;
        pub mod pallet_author_slot_filter;
        pub mod pallet_parachain_staking;
    } else {
        // Currently the benchmark does yield an invalid weight implementation
        // pub mod pallet_grandpa;
    }
}

pub mod frame_system;
pub mod orml_currencies;
pub mod orml_tokens;
pub mod pallet_balances;
pub mod pallet_bounties;
pub mod pallet_collective;
pub mod pallet_democracy;
pub mod pallet_identity;
pub mod pallet_membership;
pub mod pallet_multisig;
pub mod pallet_preimage;
pub mod pallet_proxy;
pub mod pallet_scheduler;
pub mod pallet_timestamp;
pub mod pallet_treasury;
pub mod pallet_utility;
pub mod pallet_vesting;
