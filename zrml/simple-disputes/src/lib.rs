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

#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod mock;
mod simple_disputes_pallet_api;
mod tests;

pub use pallet::*;
pub use simple_disputes_pallet_api::SimpleDisputesPalletApi;

#[frame_support::pallet]
mod pallet {
    use crate::SimpleDisputesPalletApi;
    use core::{cmp, marker::PhantomData};
    use frame_support::{
        dispatch::DispatchResult,
        log,
        pallet_prelude::{ConstU32, StorageMap, ValueQuery, Weight},
        storage::with_transaction,
        traits::{Currency, Get, Hooks, IsType},
        BoundedVec, PalletId, Twox64Concat,
    };
    use sp_runtime::{
        traits::{Saturating, Zero},
        DispatchError, TransactionOutcome,
    };
    use zeitgeist_primitives::{
        traits::{DisputeApi, DisputeResolutionApi},
        types::{Market, MarketDispute, MarketDisputeMechanism, MarketStatus, OutcomeReport},
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    type BalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type CurrencyOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Currency;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub(crate) type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;
    pub type CacheSize = ConstU32<64>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type DisputeResolution: DisputeResolutionApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
            MarketId = MarketIdOf<Self>,
            Moment = MomentOf<Self>,
        >;

        /// The identifier of individual markets.
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;

        /// The pallet identifier.
        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 1. Any resolution must either have a `Disputed` or `Reported` market status
        /// 2. If status is `Disputed`, then at least one dispute must exist
        InvalidMarketStatus,
        /// On dispute or resolution, someone tried to pass a non-simple-disputes market type
        MarketDoesNotHaveSimpleDisputesMechanism,
        StorageOverflow,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// Custom addition block initialization logic wasn't successful
        BadOnInitialize,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_initialize(now: T::BlockNumber) -> Weight {
            let mut total_weight: Weight = 0u64;

            let _ = with_transaction(|| {
                let resolve = Self::resolution_manager(now, |market_id, market| {
                    let weight = T::DisputeResolution::resolve(market_id, &market)?;
                    total_weight = total_weight.saturating_add(weight);
                    Ok(())
                });

                match resolve {
                    Err(err) => {
                        Self::deposit_event(Event::BadOnInitialize);
                        log::error!(
                            "Simple Disputes: Block {:?} was not initialized. Error: {:?}",
                            now,
                            err
                        );
                        TransactionOutcome::Rollback(err.into())
                    }
                    Ok(_) => TransactionOutcome::Commit(Ok(())),
                }
            });

            // TODO fix weight calculation
            total_weight
        }
    }

    impl<T> DisputeApi for Pallet<T>
    where
        T: Config,
    {
        type AccountId = T::AccountId;
        type Balance = BalanceOf<T>;
        type BlockNumber = T::BlockNumber;
        type MarketId = MarketIdOf<T>;
        type Moment = MomentOf<T>;
        type Origin = T::Origin;

        fn on_dispute(
            disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            market_id: &Self::MarketId,
            market: &Market<Self::AccountId, Self::BlockNumber, MomentOf<T>>,
        ) -> DispatchResult {
            if market.dispute_mechanism != MarketDisputeMechanism::SimpleDisputes {
                return Err(Error::<T>::MarketDoesNotHaveSimpleDisputesMechanism.into());
            }
            Self::remove_last_dispute_from_market_ids_per_dispute_block(&disputes, &market_id)?;
            let curr_block_num = <frame_system::Pallet<T>>::block_number();
            // each dispute resets dispute_duration
            let dispute_duration_ends_at_block =
                curr_block_num.saturating_add(market.deadlines.dispute_duration);
            <MarketIdsPerDisputeBlock<T>>::try_mutate(dispute_duration_ends_at_block, |ids| {
                ids.try_push(*market_id).map_err(|_| <Error<T>>::StorageOverflow)
            })?;
            Ok(())
        }

        fn on_resolution(
            disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            _: &Self::MarketId,
            market: &Market<Self::AccountId, Self::BlockNumber, MomentOf<T>>,
        ) -> Result<Option<OutcomeReport>, DispatchError> {
            if market.dispute_mechanism != MarketDisputeMechanism::SimpleDisputes {
                return Err(Error::<T>::MarketDoesNotHaveSimpleDisputesMechanism.into());
            }
            if market.status != MarketStatus::Disputed {
                return Err(Error::<T>::InvalidMarketStatus.into());
            }
            if let Some(last_dispute) = disputes.last() {
                Ok(Some(last_dispute.outcome.clone()))
            } else {
                Err(Error::<T>::InvalidMarketStatus.into())
            }
        }
    }

    impl<T> SimpleDisputesPalletApi for Pallet<T> where T: Config {}

    impl<T> Pallet<T>
    where
        T: Config,
    {
        pub(crate) fn resolution_manager<F>(
            now: T::BlockNumber,
            mut cb: F,
        ) -> Result<Weight, DispatchError>
        where
            F: FnMut(
                &MarketIdOf<T>,
                &Market<T::AccountId, T::BlockNumber, MomentOf<T>>,
            ) -> DispatchResult,
        {
            // Resolve any disputed markets.
            let market_ids_per_dispute_block = MarketIdsPerDisputeBlock::<T>::get(now);
            for id in market_ids_per_dispute_block.iter() {
                if let Ok(market) = T::MarketCommons::market(id) {
                    // the resolved check is required, because of admin_move_market_to_resolved
                    // only call `on_resolution` when admin_move_market_to_resolved not executed
                    if market.status != MarketStatus::Resolved {
                        cb(id, &market)?;
                    }
                } else {
                    // this is useful for admin_destroy_market
                    // because a market could be destroyed before,
                    // so only remove the id from MarketIdsPerDisputeBlock
                    log::info!(
                        "Simple Disputes: Market {:?} not found. This can happen when the market \
                         was destroyed.",
                        id
                    );
                }
            }
            MarketIdsPerDisputeBlock::<T>::remove(now);

            // TODO: fix weight calculation
            Ok(Weight::zero())
        }

        fn remove_last_dispute_from_market_ids_per_dispute_block(
            disputes: &[MarketDispute<T::AccountId, T::BlockNumber>],
            market_id: &MarketIdOf<T>,
        ) -> DispatchResult {
            if let Some(last_dispute) = disputes.last() {
                let market = T::MarketCommons::market(market_id)?;
                let dispute_duration_ends_at_block =
                    last_dispute.at.saturating_add(market.deadlines.dispute_duration);
                MarketIdsPerDisputeBlock::<T>::mutate(dispute_duration_ends_at_block, |ids| {
                    remove_item::<MarketIdOf<T>, _>(ids, market_id);
                });
            }
            Ok(())
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    /// A mapping of market identifiers to the block they were disputed at.
    /// A market only ends up here if it was disputed.
    #[pallet::storage]
    pub type MarketIdsPerDisputeBlock<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::BlockNumber,
        BoundedVec<MarketIdOf<T>, CacheSize>,
        ValueQuery,
    >;

    fn remove_item<I: cmp::PartialEq, G>(items: &mut BoundedVec<I, G>, item: &I) {
        if let Some(pos) = items.iter().position(|i| i == item) {
            items.swap_remove(pos);
        }
    }
}
