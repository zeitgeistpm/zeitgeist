// Copyright 2025 Forecasting Technologies LTD.
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

//! Helper modules that expose the legacy collator snapshot keys for each runtime.

#[cfg(feature = "parachain")]
pub mod battery_station {
    use pallet_parachain_staking::migrations::{
        LegacyAtStakeMigrationKey, LegacyAtStakeMigrationList,
    };

    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/migrations/battery_station_legacy_keys.rs"));

    pub struct BatteryStationLegacyKeys;

    impl LegacyAtStakeMigrationList for BatteryStationLegacyKeys {
        const TOTAL_KEYS: u32 = BATTERY_STATION_LEGACY_KEYS.len() as u32;

        fn get_key(index: u32) -> Option<LegacyAtStakeMigrationKey> {
            BATTERY_STATION_LEGACY_KEYS.get(index as usize).copied()
        }
    }
}

#[cfg(feature = "parachain")]
pub mod zeitgeist {
    use pallet_parachain_staking::migrations::{
        LegacyAtStakeMigrationKey, LegacyAtStakeMigrationList,
    };

    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/migrations/zeitgeist_legacy_keys.rs"));

    pub struct ZeitgeistLegacyKeys;

    impl LegacyAtStakeMigrationList for ZeitgeistLegacyKeys {
        const TOTAL_KEYS: u32 = ZEITGEIST_LEGACY_KEYS.len() as u32;

        fn get_key(index: u32) -> Option<LegacyAtStakeMigrationKey> {
            ZEITGEIST_LEGACY_KEYS.get(index as usize).copied()
        }
    }
}
