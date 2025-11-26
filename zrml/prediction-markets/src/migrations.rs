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

pub mod mbm {
    use crate::{Config, LastTimeFrame, MarketIdsPerCloseTimeFrame, MarketIdOf, TimeFrame};
    use alloc::vec::Vec;
    use frame_support::{
        migrations::{SteppedMigration, SteppedMigrationError},
        pallet_prelude::ConstU32,
        traits::Get,
        weights::{Weight, WeightMeter},
        BoundedVec,
    };
    use sp_runtime::codec::{Decode, Encode, MaxEncodedLen};
    use core::marker::PhantomData;

    /// Block time (ms) used before the AsyncBacking upgrade halved block time.
    pub const PREVIOUS_MILLISECS_PER_BLOCK: u32 = 12_000;
    /// Ratio between the previous and current block times, used to rescale timestamp frames.
    pub const TIME_FRAME_SCALE_FACTOR: TimeFrame =
        PREVIOUS_MILLISECS_PER_BLOCK as TimeFrame
            / zeitgeist_primitives::constants::MILLISECS_PER_BLOCK as TimeFrame;

    /// Target pallet storage version after migration.
    const TARGET_STORAGE_VERSION: u16 = 9;

    #[derive(Clone, Encode, Decode, MaxEncodedLen, PartialEq, Eq, Debug)]
    pub struct Cursor {
        /// Whether `LastTimeFrame` was already rescaled.
        pub rescaled_last_time_frame: bool,
        /// The time frame key to start from on the next iteration.
        pub current_time_frame: Option<TimeFrame>,
        /// Offset into the current time frame's market list.
        pub offset: u32,
        /// Highest source time frame to process (pre-migration space).
        pub max_source_time_frame: Option<TimeFrame>,
    }

    /// Multi-block migration that rescales timestamp-based close caches after the block time change.
    pub struct TimeFrameRescaleMigration<T>(PhantomData<T>);

    impl<T: Config> TimeFrameRescaleMigration<T> {
        const IDENT: &'static [u8] = b"pm-timeframe-rescale-v9";

        fn db_weight() -> frame_support::weights::RuntimeDbWeight {
            T::DbWeight::get()
        }

        fn weight_or_insufficient(
            meter: &mut WeightMeter,
            weight: Weight,
        ) -> Result<(), SteppedMigrationError> {
            meter
                .try_consume(weight)
                .map_err(|_| SteppedMigrationError::InsufficientWeight { required: weight })
        }

        fn rescale_last_time_frame(
            meter: &mut WeightMeter,
        ) -> Result<(), SteppedMigrationError> {
            Self::weight_or_insufficient(meter, Self::db_weight().reads_writes(1, 1))?;
            if let Some(last_time_frame) = LastTimeFrame::<T>::get() {
                LastTimeFrame::<T>::set(Some(
                    last_time_frame.saturating_mul(TIME_FRAME_SCALE_FACTOR),
                ));
            }
            Ok(())
        }

        fn insert_with_shift(
            market_id: MarketIdOf<T>,
            mut target_time_frame: TimeFrame,
        ) {
            loop {
                let mut bucket = MarketIdsPerCloseTimeFrame::<T>::get(target_time_frame);
                if bucket.contains(&market_id) {
                    return;
                }
                if bucket.try_push(market_id).is_ok() {
                    MarketIdsPerCloseTimeFrame::<T>::insert(target_time_frame, bucket);
                    return;
                }
                target_time_frame = target_time_frame.saturating_add(1);
            }
        }

        fn next_start_time_frame(from: TimeFrame, max: TimeFrame) -> Option<TimeFrame> {
            MarketIdsPerCloseTimeFrame::<T>::iter()
                .find(|(tf, _)| *tf >= from && *tf <= max)
                .map(|(tf, _)| tf)
        }
    }

    impl<T: Config> SteppedMigration for TimeFrameRescaleMigration<T> {
        type Cursor = Cursor;
        type Identifier = BoundedVec<u8, ConstU32<64>>;

        fn id() -> Self::Identifier {
            BoundedVec::try_from(Self::IDENT.to_vec()).expect("fits in Identifier bound; qed")
        }

        fn step(
            mut cursor: Option<Self::Cursor>,
            meter: &mut WeightMeter,
        ) -> Result<Option<Self::Cursor>, SteppedMigrationError> {
            // If storage version already bumped, skip.
            if frame_support::traits::StorageVersion::get::<crate::Pallet<T>>()
                >= frame_support::traits::StorageVersion::new(TARGET_STORAGE_VERSION)
            {
                return Ok(None);
            }

            let mut state = cursor.take().unwrap_or(Cursor {
                rescaled_last_time_frame: false,
                current_time_frame: None,
                offset: 0,
                max_source_time_frame: MarketIdsPerCloseTimeFrame::<T>::iter()
                    .last()
                    .map(|(tf, _)| tf),
            });

            if !state.rescaled_last_time_frame {
                Self::rescale_last_time_frame(meter)?;
                state.rescaled_last_time_frame = true;
            }

            let db_weight = Self::db_weight();

            let max_source = match state.max_source_time_frame {
                Some(v) => v,
                None => {
                    // No sources to process; bump version and exit.
                    Self::weight_or_insufficient(meter, db_weight.writes(1))?;
                    frame_support::traits::StorageVersion::new(TARGET_STORAGE_VERSION)
                        .put::<crate::Pallet<T>>();
                    return Ok(None);
                }
            };

            let mut start_tf = state
                .current_time_frame
                .or_else(|| Self::next_start_time_frame(0, max_source));

            while let Some(current_tf) = start_tf {
                let ids: Vec<_> = MarketIdsPerCloseTimeFrame::<T>::get(current_tf).into();

                // If offset is beyond current vec, drop the key and continue.
                if state.current_time_frame == Some(current_tf)
                    && (state.offset as usize) >= ids.len()
                {
                    Self::weight_or_insufficient(meter, db_weight.reads_writes(0, 1))?;
                    MarketIdsPerCloseTimeFrame::<T>::remove(current_tf);
                    state.current_time_frame = None;
                    state.offset = 0;
                    start_tf =
                        Self::next_start_time_frame(current_tf.saturating_add(1), max_source);
                    continue;
                }

                let mut idx: usize = if state.current_time_frame == Some(current_tf) {
                    state.offset as usize
                } else {
                    0
                };

                while idx < ids.len() {
                    // Each item roughly accounts for: read source bucket (already read),
                    // insert target bucket (read+write), write back source (later).
                    let weight_needed = db_weight.reads_writes(3, 3);
                    if meter.try_consume(weight_needed).is_err() {
                        // Account for writing back the remaining slice.
                        Self::weight_or_insufficient(meter, db_weight.writes(1))?;
                        let remaining = ids[idx..].to_vec();
                        if remaining.is_empty() {
                            MarketIdsPerCloseTimeFrame::<T>::remove(current_tf);
                        } else {
                            let bounded = BoundedVec::try_from(remaining)
                                .map_err(|_| SteppedMigrationError::Failed)?;
                            MarketIdsPerCloseTimeFrame::<T>::insert(current_tf, bounded);
                        }
                        state.current_time_frame = Some(current_tf);
                        state.offset = idx as u32;
                        return Ok(Some(state));
                    }

                    let market_id = ids[idx];
                    let target_tf = current_tf.saturating_mul(TIME_FRAME_SCALE_FACTOR);
                    Self::insert_with_shift(market_id, target_tf);
                    idx = idx.saturating_add(1);
                }

                // Finished this time frame; remove it.
                Self::weight_or_insufficient(meter, db_weight.writes(1))?;
                MarketIdsPerCloseTimeFrame::<T>::remove(current_tf);

                // Move to next available key.
                state.current_time_frame = None;
                state.offset = 0;
                start_tf = Self::next_start_time_frame(current_tf.saturating_add(1), max_source);
            }

            // No more entries; bump storage version.
            Self::weight_or_insufficient(meter, db_weight.writes(1))?;
            frame_support::traits::StorageVersion::new(TARGET_STORAGE_VERSION)
                .put::<crate::Pallet<T>>();
            Ok(None)
        }
    }
}

#[cfg(all(test, feature = "mock"))]
mod tests {
    use super::mbm::{TimeFrameRescaleMigration, TIME_FRAME_SCALE_FACTOR};
    use crate::{mock::*, CacheSize, LastTimeFrame, MarketIdsPerCloseTimeFrame};
    use frame_support::{
        migrations::SteppedMigration,
        traits::StorageVersion,
        weights::{Weight, WeightMeter},
        BoundedVec,
    };

    #[test]
    fn migration_rescales_time_frames() {
        ExtBuilder::default().build().execute_with(|| {
            StorageVersion::new(8).put::<crate::Pallet<Runtime>>();

            LastTimeFrame::<Runtime>::set(Some(5));
            MarketIdsPerCloseTimeFrame::<Runtime>::insert(
                10,
                BoundedVec::<u128, CacheSize>::try_from(vec![1u128]).unwrap(),
            );

            let mut cursor = None;
            // Give plenty of weight to finish in one go.
            let mut meter = WeightMeter::with_limit(Weight::from_parts(u64::MAX, u64::MAX));
            cursor =
                TimeFrameRescaleMigration::<Runtime>::step(cursor, &mut meter).unwrap();
            assert!(cursor.is_none());

            assert_eq!(StorageVersion::get::<crate::Pallet<Runtime>>(), StorageVersion::new(9));
            assert_eq!(
                LastTimeFrame::<Runtime>::get(),
                Some(5 * TIME_FRAME_SCALE_FACTOR)
            );
            assert_eq!(
                MarketIdsPerCloseTimeFrame::<Runtime>::get(10),
                BoundedVec::<u128, CacheSize>::new()
            );
            assert_eq!(
                MarketIdsPerCloseTimeFrame::<Runtime>::get(10 * TIME_FRAME_SCALE_FACTOR),
                BoundedVec::<u128, CacheSize>::try_from(vec![1u128]).unwrap(),
            );
        });
    }
}
