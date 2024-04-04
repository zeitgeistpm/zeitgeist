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
use sp_runtime::{traits::CheckedDiv, SaturatedConversion};

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
                            uneligible_index: old_pool_item
                                .joined_at
                                .checked_div(&T::InflationPeriod::get())
                                // because inflation period is not zero checked_div is safe
                                .unwrap_or(0u64.saturated_into::<BlockNumberOf<T>>()),
                            // using old_pool_item.stake leads to all joins in period 24
                            // to be uneligible, which is exactly what we want
                            // if using zero, all joins of period 24 would be eligible,
                            // but only joins in 23 waited the full period yet
                            // to understand the calculation of the eligible stake look into handle_inflation
                            uneligible_stake: old_pool_item.stake,
                        })
                        .collect::<Vec<_>>(),
                )
            })
        });
        match res {
            Ok(_) => log::info!("MigrateCourtPoolItems: Success!"),
            Err(e) => log::error!("MigrateCourtPoolItems: Error: {:?}", e),
        }

        total_weight = total_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));

        StorageVersion::new(COURT_NEXT_STORAGE_VERSION).put::<Court<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("MigrateCourtPoolItems: Done!");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        log::info!("MigrateCourtPoolItems: Preparing to migrate old pool items...");
        let court_pool = CourtPool::<T>::get();
        log::info!(
            "MigrateCourtPoolItems: pre-upgrade executed. Migrating {:?} pool items",
            court_pool.len()
        );
        Ok(court_pool.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(previous_state: Vec<u8>) -> Result<(), &'static str> {
        let old_court_pool: OldCourtPoolOf<T> =
            OldCourtPoolOf::<T>::decode(&mut previous_state.as_slice())
                .map_err(|_| "MigrateCourtPoolItems: failed to decode old court pool")?;
        let new_court_pool = crate::CourtPool::<T>::get();
        assert_eq!(old_court_pool.len(), new_court_pool.len());
        old_court_pool.iter().zip(new_court_pool.iter()).try_for_each(
            |(old, new)| -> Result<(), &'static str> {
                assert_eq!(old.stake, new.stake);
                assert_eq!(old.court_participant, new.court_participant);
                assert_eq!(old.consumed_stake, new.consumed_stake);
                assert_eq!(old.joined_at, new.joined_at);
                let uneligible_index = old
                    .joined_at
                    .checked_div(&T::InflationPeriod::get())
                    .ok_or("MigrateCourtPoolItems: failed to divide by inflation period")?;
                assert_eq!(new.uneligible_index, uneligible_index);
                assert_eq!(new.uneligible_stake, old.stake);

                Ok(())
            },
        )?;
        log::info!(
            "MigrateCourtPoolItems: post-upgrade executed. Migrated {:?} pool items",
            new_court_pool.len()
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
            let inflation_period = <Runtime as Config>::InflationPeriod::get();
            assert_eq!(inflation_period, 20u64);
            let joined_at_0 = 461;
            let joined_at_1 = 481;
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
                    uneligible_index: 23,
                    uneligible_stake: stake_0,
                },
                CourtPoolItemOf::<Runtime> {
                    stake: stake_1,
                    court_participant,
                    consumed_stake,
                    joined_at: joined_at_1,
                    uneligible_index: 24,
                    uneligible_stake: stake_1,
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
                    uneligible_index: 23,
                    uneligible_stake: 1,
                },
                CourtPoolItemOf::<Runtime> {
                    stake: 8,
                    court_participant: 9,
                    consumed_stake: 10,
                    joined_at: 11,
                    uneligible_index: 24,
                    uneligible_stake: 8,
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
