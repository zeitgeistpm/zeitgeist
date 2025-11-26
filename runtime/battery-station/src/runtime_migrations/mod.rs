// Copyright 2022-2025 Forecasting Technologies LTD.
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

use frame_support::migrations::MigrationId;
use pallet_parachain_staking::migrations::{
    LegacyAtStakeCursorMigration, LegacyAtStakeMigrationKey, LegacyAtStakeMigrationList,
    LEGACY_AT_STAKE_MIGRATION_ID_LEN,
};

mod legacy_keys;
use legacy_keys::BATTERY_STATION_LEGACY_KEYS;

const BATTERY_STATION_MIGRATION_BATCH: u32 = 2_000;

pub struct BatteryStationLegacyKeys;

impl LegacyAtStakeMigrationList for BatteryStationLegacyKeys {
    const TOTAL_KEYS: u32 = BATTERY_STATION_LEGACY_KEYS.len() as u32;
    const IDENTIFIER: MigrationId<{ LEGACY_AT_STAKE_MIGRATION_ID_LEN }> =
        MigrationId { pallet_id: *b"bstk", version_from: 0, version_to: 1 };

    fn get_key(index: u32) -> Option<LegacyAtStakeMigrationKey> {
        BATTERY_STATION_LEGACY_KEYS.get(index as usize).copied()
    }
}

pub type LegacyMigrations = (
    LegacyAtStakeCursorMigration<
        crate::Runtime,
        BatteryStationLegacyKeys,
        BATTERY_STATION_MIGRATION_BATCH,
    >,
);
