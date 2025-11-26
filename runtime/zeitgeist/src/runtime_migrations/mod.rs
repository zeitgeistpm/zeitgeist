use frame_support::migrations::MigrationId;
use pallet_parachain_staking::migrations::{
    LegacyAtStakeCursorMigration, LegacyAtStakeMigrationKey, LegacyAtStakeMigrationList,
    LEGACY_AT_STAKE_MIGRATION_ID_LEN,
};

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
);
