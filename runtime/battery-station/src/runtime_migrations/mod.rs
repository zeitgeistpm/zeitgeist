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
