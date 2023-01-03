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

// We use these utilities to prevent having to make the swaps pallet a dependency of
// prediciton-markets. The calls are based on the implementation of `StorageVersion`, found here:
// https://github.com/paritytech/substrate/blob/bc7a1e6c19aec92bfa247d8ca68ec63e07061032/frame/support/src/traits/metadata.rs#L168-L230
// and previous migrations.

extern crate alloc;

use crate::{types::*, Config, Pallet as GDPallet, *};
use frame_support::{
    dispatch::Weight,
    log,
    pallet_prelude::PhantomData,
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
};
use sp_runtime::traits::Saturating;

#[cfg(feature = "try-runtime")]
use alloc::string::ToString;
#[cfg(feature = "try-runtime")]
use scale_info::prelude::format;

const GD_REQUIRED_STORAGE_VERSION: u16 = 0;
const GD_NEXT_STORAGE_VERSION: u16 = 1;

pub struct ModifyGlobalDisputesStructures<T>(PhantomData<T>);

impl<T: Config + zrml_market_commons::Config> OnRuntimeUpgrade
    for ModifyGlobalDisputesStructures<T>
{
    fn on_runtime_upgrade() -> Weight
    where
        T: Config,
    {
        let mut total_weight = T::DbWeight::get().reads(1);
        let gd_version = StorageVersion::get::<GDPallet<T>>();
        if gd_version != GD_REQUIRED_STORAGE_VERSION {
            log::info!(
                "ModifyGlobalDisputesStructures: global disputes version is {:?}, require {:?};",
                gd_version,
                GD_REQUIRED_STORAGE_VERSION,
            );
            return total_weight;
        }
        log::info!("ModifyGlobalDisputesStructures: Starting...");

        for (market_id, winner_info) in crate::Winners::<T>::iter() {
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));

            let owners = winner_info.outcome_info.owners;
            let owners_len = owners.len();
            let possession = match owners_len {
                0usize => None,
                1usize => Some(Possession::Paid {
                    owner: owners
                        .get(0)
                        .expect("Owners len is 1, but could not get this owner.")
                        .clone(),
                    fee: T::VotingOutcomeFee::get(),
                }),
                _ => Some(Possession::Shared { owners }),
            };

            let outcome_info =
                OutcomeInfo { outcome_sum: winner_info.outcome_info.outcome_sum, possession };
            let gd_info = GDInfo {
                winner_outcome: winner_info.outcome,
                outcome_info,
                status: GDStatus::Finished,
            };
            crate::GlobalDisputesInfo::<T>::insert(market_id, gd_info);
            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        }

        // Look here https://github.com/paritytech/substrate/blob/7ba1ef526fbcc224d3bd446c83ee66a83cacec49/frame/assets/src/migration.rs#L69-L79
        let mut translated = 0u64;
        Outcomes::<T>::translate::<OldOutcomeInfo<BalanceOf<T>, OwnerInfoOf<T>>, _>(
            |_key1, _key2, old_value| {
                translated.saturating_inc();

                let owners = old_value.owners;
                let owners_len = owners.len();
                let possession = match owners_len {
                    0usize => None,
                    1usize => Some(Possession::Paid {
                        owner: owners
                            .get(0)
                            .expect("Owners len is 1, but could not get this owner.")
                            .clone(),
                        fee: T::VotingOutcomeFee::get(),
                    }),
                    _ => Some(Possession::Shared { owners }),
                };

                let new_value = OutcomeInfo { outcome_sum: old_value.outcome_sum, possession };

                Some(new_value)
            },
        );
        log::info!("ModifyGlobalDisputesStructures: Upgraded {} outcomes.", translated);
        total_weight = total_weight
            .saturating_add(T::DbWeight::get().reads_writes(translated + 1, translated + 1));

        StorageVersion::new(GD_NEXT_STORAGE_VERSION).put::<GDPallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("ModifyGlobalDisputesStructures: Done!");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        use frame_support::traits::OnRuntimeUpgradeHelpersExt;

        let mut counter = 0_u32;
        for (market_id, winner_info) in crate::Winners::<T>::iter() {
            Self::set_temp_storage(winner_info, &format!("{:?}", market_id));

            counter = counter.saturating_add(1_u32);
        }
        let counter_key = "counter_key".to_string();
        Self::set_temp_storage(counter, &counter_key);
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        use frame_support::traits::OnRuntimeUpgradeHelpersExt;

        let mut markets_count = 0_u32;
        let old_counter_key = "counter_key".to_string();
        for (market_id, gd_info) in crate::GlobalDisputesInfo::<T>::iter() {
            let GDInfo { winner_outcome, outcome_info, status } = gd_info;

            let market_id_str = format!("{:?}", market_id);

            let winner_info: OldWinnerInfo<BalanceOf<T>, OwnerInfoOf<T>> =
                Self::get_temp_storage(&market_id_str)
                    .unwrap_or_else(|| panic!("old value not found for market id {:?}", market_id));

            assert_eq!(winner_outcome, winner_info.outcome);
            assert_eq!(status, GDStatus::Finished);

            let owners = winner_info.outcome_info.owners;
            let owners_len = owners.len();

            let possession = match owners_len {
                0usize => None,
                1usize => Some(Possession::Paid {
                    owner: owners
                        .get(0)
                        .expect("Owners len is 1, but could not get this owner.")
                        .clone(),
                    fee: T::VotingOutcomeFee::get(),
                }),
                _ => Some(Possession::Shared { owners }),
            };

            let outcome_info_expected =
                OutcomeInfo { outcome_sum: winner_info.outcome_info.outcome_sum, possession };
            assert_eq!(outcome_info, outcome_info_expected);

            markets_count += 1_u32;
        }
        let old_markets_count: u32 =
            Self::get_temp_storage(&old_counter_key).expect("old counter key storage not found");
        assert_eq!(markets_count, old_markets_count);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{ExtBuilder, Runtime, ALICE, BOB};
    use frame_support::{
        migration::{get_storage_value, put_storage_value},
        BoundedVec,
    };
    use sp_runtime::traits::SaturatedConversion;
    use zeitgeist_primitives::{
        constants::mock::VotingOutcomeFee,
        types::{MarketId, OutcomeReport},
    };

    const GLOBAL_DISPUTES: &[u8] = b"GlobalDisputes";
    const GD_OUTCOMES: &[u8] = b"Outcomes";

    type OldOutcomeInfoOf<Runtime> = OldOutcomeInfo<BalanceOf<Runtime>, OwnerInfoOf<Runtime>>;

    #[test]
    fn on_runtime_upgrade_increments_the_storage_versions() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_chain();
            ModifyGlobalDisputesStructures::<Runtime>::on_runtime_upgrade();
            let gd_version = StorageVersion::get::<GDPallet<Runtime>>();
            assert_eq!(gd_version, GD_NEXT_STORAGE_VERSION);
        });
    }

    #[test]
    fn on_runtime_sets_new_global_disputes_storage_paid() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_chain();

            let market_id = 0u128;

            let outcome_sum = 42u128.saturated_into::<BalanceOf<Runtime>>();
            let owners = BoundedVec::try_from(vec![ALICE]).unwrap();

            let outcome_info = OldOutcomeInfo { outcome_sum, owners };
            let outcome = OutcomeReport::Categorical(42u16);
            let winner_info =
                OldWinnerInfo { outcome: outcome.clone(), outcome_info, is_finished: true };

            crate::Winners::<Runtime>::insert(market_id, winner_info);

            ModifyGlobalDisputesStructures::<Runtime>::on_runtime_upgrade();

            let possession = Some(Possession::Paid { owner: ALICE, fee: VotingOutcomeFee::get() });

            let new_outcome_info = OutcomeInfo { outcome_sum, possession };

            let expected = GDInfo {
                winner_outcome: outcome,
                outcome_info: new_outcome_info,
                status: GDStatus::Finished,
            };

            let actual = crate::GlobalDisputesInfo::<Runtime>::get(market_id).unwrap();
            assert_eq!(expected, actual);
        });
    }

    #[test]
    fn on_runtime_sets_new_global_disputes_storage_shared() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_chain();

            let market_id = 0u128;

            let outcome_sum = 42u128.saturated_into::<BalanceOf<Runtime>>();
            let owners = BoundedVec::try_from(vec![ALICE, BOB]).unwrap();

            let outcome_info = OldOutcomeInfo { outcome_sum, owners: owners.clone() };
            let outcome = OutcomeReport::Categorical(42u16);
            let winner_info =
                OldWinnerInfo { outcome: outcome.clone(), outcome_info, is_finished: true };

            crate::Winners::<Runtime>::insert(market_id, winner_info);

            ModifyGlobalDisputesStructures::<Runtime>::on_runtime_upgrade();

            let possession = Some(Possession::Shared { owners });

            let new_outcome_info = OutcomeInfo { outcome_sum, possession };

            let expected = GDInfo {
                winner_outcome: outcome,
                outcome_info: new_outcome_info,
                status: GDStatus::Finished,
            };

            let actual = crate::GlobalDisputesInfo::<Runtime>::get(market_id).unwrap();
            assert_eq!(expected, actual);
        });
    }

    #[test]
    fn on_runtime_sets_new_outcomes_storage_value_shared() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_chain();

            let outcome = OutcomeReport::Categorical(0u16);
            let hash =
                crate::Outcomes::<Runtime>::hashed_key_for::<MarketId, OutcomeReport>(0, outcome);

            let outcome_sum = 42u128.saturated_into::<BalanceOf<Runtime>>();
            let owners = BoundedVec::try_from(vec![ALICE, BOB]).unwrap();

            let outcome_info = OldOutcomeInfo { outcome_sum, owners: owners.clone() };

            put_storage_value::<OldOutcomeInfoOf<Runtime>>(
                GLOBAL_DISPUTES,
                GD_OUTCOMES,
                &hash,
                outcome_info,
            );

            ModifyGlobalDisputesStructures::<Runtime>::on_runtime_upgrade();

            let possession = Some(Possession::Shared { owners });
            let expected = OutcomeInfo { outcome_sum, possession };

            let actual = frame_support::migration::get_storage_value::<OutcomeInfoOf<Runtime>>(
                GLOBAL_DISPUTES,
                GD_OUTCOMES,
                &hash,
            )
            .unwrap();
            assert_eq!(expected, actual);
        });
    }

    #[test]
    fn on_runtime_sets_new_outcomes_storage_value_paid() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_chain();

            let outcome = OutcomeReport::Categorical(0u16);
            let hash =
                crate::Outcomes::<Runtime>::hashed_key_for::<MarketId, OutcomeReport>(0, outcome);

            let outcome_sum = 42u128.saturated_into::<BalanceOf<Runtime>>();
            let owners = BoundedVec::try_from(vec![ALICE]).unwrap();

            let outcome_info = OldOutcomeInfo { outcome_sum, owners };

            put_storage_value::<OldOutcomeInfoOf<Runtime>>(
                GLOBAL_DISPUTES,
                GD_OUTCOMES,
                &hash,
                outcome_info,
            );

            ModifyGlobalDisputesStructures::<Runtime>::on_runtime_upgrade();

            let possession = Some(Possession::Paid { owner: ALICE, fee: VotingOutcomeFee::get() });
            let expected = OutcomeInfo { outcome_sum, possession };

            let actual = frame_support::migration::get_storage_value::<OutcomeInfoOf<Runtime>>(
                GLOBAL_DISPUTES,
                GD_OUTCOMES,
                &hash,
            )
            .unwrap();
            assert_eq!(expected, actual);
        });
    }

    #[test]
    fn on_runtime_is_noop_if_versions_are_not_correct() {
        ExtBuilder::default().build().execute_with(|| {
            // storage migration already executed (storage version is incremented already)
            StorageVersion::new(GD_NEXT_STORAGE_VERSION).put::<GDPallet<Runtime>>();

            let outcome = OutcomeReport::Categorical(0u16);
            let hash =
                crate::Outcomes::<Runtime>::hashed_key_for::<MarketId, OutcomeReport>(0, outcome);

            let outcome_info = OldOutcomeInfo {
                outcome_sum: 0u128.saturated_into::<BalanceOf<Runtime>>(),
                owners: BoundedVec::try_from(vec![ALICE]).unwrap(),
            };

            put_storage_value::<OldOutcomeInfoOf<Runtime>>(
                GLOBAL_DISPUTES,
                GD_OUTCOMES,
                &hash,
                outcome_info,
            );

            ModifyGlobalDisputesStructures::<Runtime>::on_runtime_upgrade();

            // no changes should have been made, because the storage version was already incremented
            assert!(
                get_storage_value::<OutcomeInfoOf<Runtime>>(GLOBAL_DISPUTES, GD_OUTCOMES, &hash)
                    .is_none()
            );
        });
    }

    fn set_up_chain() {
        StorageVersion::new(GD_REQUIRED_STORAGE_VERSION).put::<GDPallet<Runtime>>();
    }
}
