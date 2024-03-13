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

use crate::{
    AccountIdOf, BalanceOf, BlockNumberOf, Config, CourtPoolItemOf, CourtPoolOf, Pallet as Court,
};
#[cfg(feature = "try-runtime")]
use alloc::vec;
use alloc::vec::Vec;
use core::marker::PhantomData;
use frame_support::{
    log,
    pallet_prelude::{StorageVersion, ValueQuery, Weight},
    traits::{Get, OnRuntimeUpgrade},
    BoundedVec,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[derive(Decode, Encode, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq, Eq)]
pub struct OldCourtPoolItem<AccountId, Balance, BlockNumber> {
    pub stake: Balance,
    pub court_participant: AccountId,
    pub consumed_stake: Balance,
    pub joined_at: BlockNumber,
}

type OldCourtPoolItemOf<T> = OldCourtPoolItem<AccountIdOf<T>, BalanceOf<T>, BlockNumberOf<T>>;
type OldCourtPoolOf<T> = BoundedVec<OldCourtPoolItemOf<T>, <T as Config>::MaxCourtParticipants>;

#[frame_support::storage_alias]
pub(crate) type CourtPool<T: Config> = StorageValue<Court<T>, OldCourtPoolOf<T>, ValueQuery>;

const COURT_REQUIRED_STORAGE_VERSION: u16 = 2;
const COURT_NEXT_STORAGE_VERSION: u16 = 3;

pub struct MigrateCourtPoolItems<T>(PhantomData<T>);

impl<T> OnRuntimeUpgrade for MigrateCourtPoolItems<T>
where
    T: Config,
{
    fn on_runtime_upgrade() -> Weight {
        let mut total_weight = T::DbWeight::get().reads(1);
        let market_commons_version = StorageVersion::get::<Court<T>>();
        if market_commons_version != COURT_REQUIRED_STORAGE_VERSION {
            log::info!(
                "MigrateCourtPoolItems: court storage version is {:?}, but {:?} is required",
                market_commons_version,
                COURT_REQUIRED_STORAGE_VERSION,
            );
            return total_weight;
        }
        log::info!("MigrateCourtPoolItems: Starting...");

        let res = crate::CourtPool::<T>::translate::<OldCourtPoolOf<T>, _>(|old_pool_opt| {
            old_pool_opt.map(|mut old_pool| {
                <CourtPoolOf<T>>::truncate_from(
                    old_pool
                        .iter_mut()
                        .map(|old_pool_item: &mut OldCourtPoolItemOf<T>| CourtPoolItemOf::<T> {
                            stake: old_pool_item.stake,
                            court_participant: old_pool_item.court_participant.clone(),
                            consumed_stake: old_pool_item.consumed_stake,
                            joined_at: old_pool_item.joined_at,
                            last_join_at: old_pool_item.joined_at,
                            pre_period_join_stake: old_pool_item.stake,
                            pre_period_join_at: old_pool_item.joined_at,
                        })
                        .collect::<Vec<_>>(),
                )
            })
        });
        match res {
            Ok(_) => log::info!("MigrateCourtPoolItems: Success!"),
            Err(e) => log::error!("MigrateCourtPoolItems: Error: {:?}", e),
        }

        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));

        StorageVersion::new(COURT_NEXT_STORAGE_VERSION).put::<Court<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("MigrateCourtPoolItems: Done!");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        log::info!("MigrateCourtPoolItems: Preparing to migrate old pool items...");
        Ok(vec![])
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_previous_state: Vec<u8>) -> Result<(), &'static str> {
        let court_pool = crate::CourtPool::<T>::get();
        log::info!(
            "MigrateCourtPoolItems: post-upgrade executed. Migrated {:?} pool items",
            court_pool.len()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{ExtBuilder, Runtime};
    use frame_support::storage_root;
    use sp_runtime::StateVersion;

    #[test]
    fn on_runtime_upgrade_increments_the_storage_version() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            MigrateCourtPoolItems::<Runtime>::on_runtime_upgrade();
            assert_eq!(StorageVersion::get::<Court<Runtime>>(), COURT_NEXT_STORAGE_VERSION);
        });
    }

    #[test]
    fn on_runtime_upgrade_works_as_expected() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            let stake_0 = 100;
            let stake_1 = 101;
            let court_participant = 200;
            let consumed_stake = 300;
            let joined_at_0 = 42;
            let joined_at_1 = 123;
            let old_court_pool = OldCourtPoolOf::<Runtime>::truncate_from(vec![
                OldCourtPoolItemOf::<Runtime> {
                    stake: stake_0,
                    court_participant,
                    consumed_stake,
                    joined_at: joined_at_0,
                },
                OldCourtPoolItemOf::<Runtime> {
                    stake: stake_1,
                    court_participant,
                    consumed_stake,
                    joined_at: joined_at_1,
                },
            ]);
            let new_court_pool = CourtPoolOf::<Runtime>::truncate_from(vec![
                CourtPoolItemOf::<Runtime> {
                    stake: stake_0,
                    court_participant,
                    consumed_stake,
                    joined_at: joined_at_0,
                    last_join_at: joined_at_0,
                    pre_period_join_stake: stake_0,
                    pre_period_join_at: joined_at_0,
                },
                CourtPoolItemOf::<Runtime> {
                    stake: stake_1,
                    court_participant,
                    consumed_stake,
                    joined_at: joined_at_1,
                    last_join_at: joined_at_1,
                    pre_period_join_stake: stake_1,
                    pre_period_join_at: joined_at_1,
                },
            ]);
            CourtPool::<Runtime>::put::<OldCourtPoolOf<Runtime>>(old_court_pool);
            // notice we use the old storage item here to find out if we added the old elements
            // decoding of the values would fail
            assert_eq!(crate::CourtPool::<Runtime>::decode_len().unwrap(), 2usize);
            MigrateCourtPoolItems::<Runtime>::on_runtime_upgrade();

            let actual = crate::CourtPool::<Runtime>::get();
            assert_eq!(actual, new_court_pool);
        });
    }

    #[test]
    fn on_runtime_upgrade_is_noop_if_versions_are_not_correct() {
        ExtBuilder::default().build().execute_with(|| {
            StorageVersion::new(COURT_NEXT_STORAGE_VERSION).put::<Court<Runtime>>();
            let court_pool = <CourtPoolOf<Runtime>>::truncate_from(vec![
                CourtPoolItemOf::<Runtime> {
                    stake: 1,
                    court_participant: 2,
                    consumed_stake: 3,
                    joined_at: 4,
                    last_join_at: 4,
                    pre_period_join_stake: 4,
                    pre_period_join_at: 4,
                },
                CourtPoolItemOf::<Runtime> {
                    stake: 8,
                    court_participant: 9,
                    consumed_stake: 10,
                    joined_at: 11,
                    last_join_at: 11,
                    pre_period_join_stake: 11,
                    pre_period_join_at: 11,
                },
            ]);
            crate::CourtPool::<Runtime>::put(court_pool);
            let tmp = storage_root(StateVersion::V1);
            MigrateCourtPoolItems::<Runtime>::on_runtime_upgrade();
            assert_eq!(tmp, storage_root(StateVersion::V1));
        });
    }

    fn set_up_version() {
        StorageVersion::new(COURT_REQUIRED_STORAGE_VERSION).put::<Court<Runtime>>();
    }
}
