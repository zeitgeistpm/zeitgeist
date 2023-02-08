// Copyright 2022 Forecasting Technologies LTD.
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

mod benchmarks;
mod crowdfund_pallet_api;
mod mock;
mod tests;
pub mod types;

pub use crowdfund_pallet_api::CrowdfundPalletApi;
pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::{
        types::{FundItemStatus, *},
        CrowdfundPalletApi,
    };
    use core::marker::PhantomData;
    use frame_support::{
        ensure, log,
        pallet_prelude::{
            DispatchError, DispatchResultWithPostInfo, MaybeSerializeDeserialize, Member,
            OptionQuery, StorageDoubleMap, StorageMap, StorageValue, TypeInfo, ValueQuery,
        },
        traits::{Currency, ExistenceRequirement, Get, IsType},
        Blake2_128Concat, PalletId, Parameter, Twox64Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use parity_scale_codec::MaxEncodedLen;
    use sp_runtime::{
        traits::{AccountIdConversion, Saturating, StaticLookup, Zero},
        DispatchResult, Percent,
    };

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[pallet::constant]
        type AppealThreshold: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type ClearLimit: Get<u32>;

        /// The pallet identifier.
        #[pallet::constant]
        type CrowdfundPalletId: Get<PalletId>;

        /// The currency implementation used to lock tokens for voting.
        type Currency: Currency<Self::AccountId>;

        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type FundItem: Parameter
            + Member
            + Ord
            + Clone
            + TypeInfo
            + MaybeSerializeDeserialize
            + Default
            + MaxEncodedLen;

        #[pallet::constant]
        type MinFunding: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type IterationLimit: Get<u32>;
    }

    pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

    pub type FundItemInfoOf<T> = FundItemInfo<BalanceOf<T>>;
    pub type BackerInfoOf<T> = BackerInfo<BalanceOf<T>>;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::storage]
    pub type Crowdfunds<T: Config> =
        StorageMap<_, Twox64Concat, FundIndex, CrowdfundInfo, OptionQuery>;

    #[pallet::storage]
    pub type NextFundIndex<T: Config> = StorageValue<_, FundIndex, ValueQuery>;

    #[pallet::storage]
    pub type FundItems<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        FundIndex,
        Blake2_128Concat,
        T::FundItem,
        FundItemInfoOf<T>,
        OptionQuery,
    >;

    #[pallet::storage]
    pub type Backers<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        AccountIdOf<T>,
        Blake2_128Concat,
        (FundIndex, T::FundItem),
        BackerInfoOf<T>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// A crowdfund item was funded.
        ItemFunded {
            who: AccountIdOf<T>,
            fund_index: FundIndex,
            item: T::FundItem,
            amount: BalanceOf<T>,
        },
        /// A backer refunded all of their funds.
        BackerFullyRefunded { backer: AccountIdOf<T>, refunded: BalanceOf<T> },
        /// A backer refunded some of their funds.
        BackerPartiallyRefunded { backer: AccountIdOf<T>, refunded: BalanceOf<T> },
        /// A crowdfund was opened.
        CrowdfundOpened { fund_index: FundIndex },
        /// A crowdfund was closed.
        CrowdfundClosed { fund_index: FundIndex },
        /// A crowdfund was cleared.
        CrowdfundFullyCleared { fund_index: FundIndex },
        /// A crowdfund was partially cleared.
        CrowdfundPartiallyCleared { fund_index: FundIndex },
    }

    #[pallet::error]
    pub enum Error<T> {
        CrowdfundNotFound,
        AmountTooLow,
        CrowdfundNotActive,
        CrowdfundNotClosed,
        FundItemNotFound,
        FundIndexOverflow,
        InvalidShare,
        InvalidFundItemStatus,
        NotFullyRefunded,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Fund an item.
        ///
        /// # Arguments
        ///
        /// - `fund_index`: The fund identifier of the crowdfund.
        /// - `item`: The item to fund.
        /// - `amount`: The amount to fund.
        ///
        /// # Weight
        ///
        /// Complexity: `O(1)`
        #[frame_support::transactional]
        #[pallet::weight(5000)]
        pub fn fund(
            origin: OriginFor<T>,
            fund_index: FundIndex,
            item: T::FundItem,
            amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            ensure!(amount >= T::MinFunding::get(), Error::<T>::AmountTooLow);
            let crowdfund_info =
                <Crowdfunds<T>>::get(&fund_index).ok_or(Error::<T>::CrowdfundNotFound)?;
            ensure!(
                crowdfund_info.status == CrowdfundStatus::Active,
                Error::<T>::CrowdfundNotActive
            );

            let fund_account = Self::crowdfund_account();
            T::Currency::transfer(&who, &fund_account, amount, ExistenceRequirement::AllowDeath)?;

            let mut fund_item = <FundItems<T>>::get(&fund_index, &item)
                .unwrap_or(FundItemInfo { raised: Zero::zero(), status: FundItemStatus::Active });
            fund_item.raised = fund_item.raised.saturating_add(amount);

            let mut backer = <Backers<T>>::get(&who, (&fund_index, &item))
                .unwrap_or(BackerInfo { amount: Zero::zero() });
            backer.amount = backer.amount.saturating_add(amount);

            <FundItems<T>>::insert(&fund_index, &item, fund_item);

            <Backers<T>>::insert(&who, (&fund_index, &item), backer);

            Self::deposit_event(Event::ItemFunded { who, fund_index, item, amount });

            Ok(Some(5000).into())
        }

        /// Refund all crowdfunds, which are in a refundable state.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, in which `n` is the number of fund items of the caller.
        #[frame_support::transactional]
        #[pallet::weight(5000)]
        pub fn refund(
            origin: OriginFor<T>,
            backer: AccountIdLookupOf<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;
            let backer = T::Lookup::lookup(backer)?;

            let mut amount = <BalanceOf<T>>::zero();
            let mut removables = Vec::new();
            for ((fund_index, item), backer_info) in
                <Backers<T>>::iter_prefix(&backer).take(T::IterationLimit::get() as usize)
            {
                if let Some(mut fund_item) = <FundItems<T>>::get(&fund_index, &item) {
                    match fund_item.status {
                        FundItemStatus::Refundable { share } => {
                            let refund_amount = share * backer_info.amount;
                            amount = amount.saturating_add(refund_amount);

                            removables.push((fund_index, item.clone()));

                            fund_item.raised = fund_item.raised.saturating_sub(backer_info.amount);
                            <FundItems<T>>::insert(&fund_index, &item, fund_item);
                        }
                        FundItemStatus::Active => (),
                    }
                } else {
                    log::error!(
                        "Fund item not found for fund index {:?} and item {:?}",
                        fund_index,
                        item
                    );
                    debug_assert!(false);
                    continue;
                }
            }

            let fund_account = Self::crowdfund_account();
            T::Currency::transfer(
                &fund_account,
                &backer,
                amount,
                ExistenceRequirement::AllowDeath,
            )?;

            for (fund_index, item) in removables {
                // active fund items are not removed, only spent and refundables
                <Backers<T>>::remove(&backer, (&fund_index, &item));
            }

            if <Backers<T>>::iter_prefix(&backer).next().is_some() {
                Self::deposit_event(Event::BackerPartiallyRefunded { backer, refunded: amount });
            } else {
                Self::deposit_event(Event::BackerFullyRefunded { backer, refunded: amount });
            }

            Ok(Some(5000).into())
        }

        /// Clear the storage of a crowdfund.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, in which `n` is the number of fund items.
        #[frame_support::transactional]
        #[pallet::weight(5000)]
        pub fn clear(origin: OriginFor<T>, fund_index: FundIndex) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;

            let crowdfund_info =
                <Crowdfunds<T>>::get(&fund_index).ok_or(Error::<T>::CrowdfundNotFound)?;
            ensure!(
                crowdfund_info.status == CrowdfundStatus::Closed,
                Error::<T>::CrowdfundNotClosed
            );

            let mut removables = Vec::new();
            for (fund_item, fund_info) in
                <FundItems<T>>::iter_prefix(&fund_index).take(T::ClearLimit::get() as usize)
            {
                ensure!(
                    matches!(fund_info.status, FundItemStatus::Refundable { .. }),
                    Error::<T>::InvalidFundItemStatus
                );
                ensure!(fund_info.raised.is_zero(), Error::<T>::NotFullyRefunded);
                removables.push(fund_item);
            }

            for fund_item in removables {
                <FundItems<T>>::remove(&fund_index, &fund_item);
            }

            if <FundItems<T>>::iter_prefix(&fund_index).next().is_some() {
                Self::deposit_event(Event::CrowdfundPartiallyCleared { fund_index });
            } else {
                <Crowdfunds<T>>::remove(&fund_index);
                Self::deposit_event(Event::CrowdfundFullyCleared { fund_index });
            }

            Ok(Some(5000).into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// The account ID of the crowdfund pallet.
        pub fn crowdfund_account() -> T::AccountId {
            T::CrowdfundPalletId::get().into_account_truncating()
        }
    }

    impl<T> CrowdfundPalletApi<AccountIdOf<T>, BalanceOf<T>, T::FundItem> for Pallet<T>
    where
        T: Config,
    {
        fn open_crowdfund() -> Result<FundIndex, DispatchError> {
            let fund_index = <NextFundIndex<T>>::get();
            let next_fund_index = fund_index.checked_add(1).ok_or(Error::<T>::FundIndexOverflow)?;
            let status = CrowdfundStatus::Active;
            let crowdfund_info = CrowdfundInfo { status };
            <Crowdfunds<T>>::insert(fund_index, crowdfund_info);
            <NextFundIndex<T>>::put(next_fund_index);
            Self::deposit_event(Event::CrowdfundOpened { fund_index });
            Ok(fund_index)
        }

        fn iter_items(
            fund_index: FundIndex,
        ) -> frame_support::storage::PrefixIterator<(T::FundItem, FundItemInfoOf<T>)> {
            <FundItems<T>>::iter_prefix(fund_index)
        }

        fn set_item_status(
            fund_index: FundIndex,
            item: &T::FundItem,
            status: FundItemStatus,
        ) -> DispatchResult {
            let mut fund_item =
                <FundItems<T>>::get(fund_index, item).ok_or(Error::<T>::FundItemNotFound)?;
            match status {
                FundItemStatus::Active => (),
                FundItemStatus::Refundable { share } => {
                    ensure!(
                        Percent::from_percent(0) <= share && share <= Percent::from_percent(100),
                        Error::<T>::InvalidShare
                    );
                }
            }
            fund_item.status = status;
            <FundItems<T>>::insert(fund_index, item, fund_item);
            Ok(())
        }

        fn close_crowdfund(fund_index: FundIndex) -> DispatchResult {
            let mut crowdfund_info =
                <Crowdfunds<T>>::get(fund_index).ok_or(Error::<T>::CrowdfundNotFound)?;
            crowdfund_info.status = CrowdfundStatus::Closed;
            <Crowdfunds<T>>::insert(fund_index, crowdfund_info);
            Self::deposit_event(Event::CrowdfundClosed { fund_index });
            Ok(())
        }

        fn get_fund_account() -> AccountIdOf<T> {
            Self::crowdfund_account()
        }
    }
}
