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
use zrml_prediction_markets::migrations::mbm::TimeFrameRescaleMigration;

mod legacy_keys;
use legacy_keys::ZEITGEIST_LEGACY_KEYS;

const ZEITGEIST_MIGRATION_BATCH: u32 = 2_000;

pub struct ZeitgeistLegacyKeys;

impl LegacyAtStakeMigrationList for ZeitgeistLegacyKeys {
    const TOTAL_KEYS: u32 = ZEITGEIST_LEGACY_KEYS.len() as u32;
    const IDENTIFIER: MigrationId<{ LEGACY_AT_STAKE_MIGRATION_ID_LEN }> =
        MigrationId { pallet_id: *b"zstk", version_from: 0, version_to: 1 };

    fn get_key(index: u32) -> Option<LegacyAtStakeMigrationKey> {
        ZEITGEIST_LEGACY_KEYS.get(index as usize).copied()
    }
}

pub type LegacyMigrations = (
    LegacyAtStakeCursorMigration<crate::Runtime, ZeitgeistLegacyKeys, ZEITGEIST_MIGRATION_BATCH>,
    TimeFrameRescaleMigration<crate::Runtime>,
);
