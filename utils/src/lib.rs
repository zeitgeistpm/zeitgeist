// Copyright 2021-2023 Centrifuge Foundation (centrifuge.io).
//
// Copyright 2023 Zeitgeist PM LLC.
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

// Ensure we're `no_std` when compiling for WebAssembly.
#![cfg_attr(not(feature = "std"), no_std)]

/// Function that initializes the frame system & Aura, so a timestamp can be set and pass validation
#[cfg(any(feature = "runtime-benchmarks", feature = "std"))]
pub fn set_block_number_timestamp<T>(block_number: T::BlockNumber, timestamp: T::Moment)
where
    T: pallet_aura::Config + frame_system::Config + pallet_timestamp::Config,
{
    use codec::Encode;
    use frame_support::traits::Hooks;
    use sp_consensus_aura::AURA_ENGINE_ID;
    use sp_runtime::{Digest, DigestItem};
    use sp_std::vec;

    let slot = timestamp / pallet_aura::Pallet::<T>::slot_duration();
    let digest = Digest { logs: vec![DigestItem::PreRuntime(AURA_ENGINE_ID, slot.encode())] };
    frame_system::Pallet::<T>::initialize(&block_number, &Default::default(), &digest);
    // NOTE: pallet aura only used in standalone mode when
    //       the `parachain` feature is disabled.
    pallet_aura::Pallet::<T>::on_initialize(block_number);
    pallet_timestamp::Pallet::<T>::set_timestamp(timestamp);
}
