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
    use crate::{types::*, CrowdfundPalletApi};
    use core::marker::PhantomData;
    use frame_support::{
        ensure, log,
        pallet_prelude::{DispatchResultWithPostInfo, OptionQuery, StorageDoubleMap, StorageMap},
        traits::{Currency, ExistenceRequirement, Get, IsType},
        Blake2_128Concat, PalletId, Twox64Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use sp_runtime::{
        traits::{AccountIdConversion, Saturating, Zero},
        DispatchResult,
    };
    use crate::types::FundItemStatus;
    use zeitgeist_primitives::types::OutcomeReport;
    use zrml_market_commons::MarketCommonsPalletApi;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[pallet::constant]
        type AppealThreshold: Get<BalanceOf<Self>>;

        /// The pallet identifier.
        #[pallet::constant]
        type CrowdfundPalletId: Get<PalletId>;

        /// The currency implementation used to lock tokens for voting.
        type Currency: Currency<Self::AccountId>;

        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// To reference the market id type.
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;

        #[pallet::constant]
        type MinFunding: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type IterationLimit: Get<u32>;
    }

    pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

    pub type FundItemInfoOf<T> = FundItemInfo<BalanceOf<T>>;
    pub type BackerInfoOf<T> = BackerInfo<BalanceOf<T>>;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::storage]
    pub type Crowdfunds<T: Config> =
        StorageMap<_, Twox64Concat, MarketIdOf<T>, CrowdfundInfo, OptionQuery>;

    #[pallet::storage]
    pub type FundItems<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        MarketIdOf<T>,
        Blake2_128Concat,
        OutcomeReport,
        FundItemInfoOf<T>,
        OptionQuery,
    >;

    #[pallet::storage]
    pub type Backers<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        AccountIdOf<T>,
        Blake2_128Concat,
        (MarketIdOf<T>, OutcomeReport),
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
            market_id: MarketIdOf<T>,
            item: OutcomeReport,
            amount: BalanceOf<T>,
        },
        /// A backer refunded all of their funds.
        AllRefunded { who: AccountIdOf<T>, amount: BalanceOf<T> },
        /// A backer refunded some of their funds.
        PartiallyRefunded { who: AccountIdOf<T>, amount: BalanceOf<T> },
    }

    #[pallet::error]
    pub enum Error<T> {
        CrowdfundNotFound,
        AmountTooLow,
        CrowdfundNotActive,
        CrowdfundNotClosed,
        FundItemNotFound,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Fund an item.
        ///
        /// # Arguments
        ///
        /// - `market_id`: The market id of the crowdfund.
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
            market_id: MarketIdOf<T>,
            item: OutcomeReport,
            amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            ensure!(amount >= T::MinFunding::get(), Error::<T>::AmountTooLow);
            let crowdfund_info =
                <Crowdfunds<T>>::get(&market_id).ok_or(Error::<T>::CrowdfundNotFound)?;
            ensure!(
                crowdfund_info.status == CrowdfundStatus::Active,
                Error::<T>::CrowdfundNotActive
            );

            let fund_account = Self::crowdfund_account();
            T::Currency::transfer(&who, &fund_account, amount, ExistenceRequirement::AllowDeath)?;

            let mut fund_item = <FundItems<T>>::get(&market_id, &item)
                .unwrap_or(FundItemInfo { raised: Zero::zero(), status: FundItemStatus::Active });
            fund_item.raised = fund_item.raised.saturating_add(amount);

            let mut backer = <Backers<T>>::get(&who, (&market_id, &item))
                .unwrap_or(BackerInfo { amount: Zero::zero() });
            backer.amount = backer.amount.saturating_add(amount);

            <FundItems<T>>::insert(&market_id, &item, fund_item);

            <Backers<T>>::insert(&who, (&market_id, &item), backer);

            Self::deposit_event(Event::ItemFunded { who, market_id, item, amount });

            Ok(Some(5000).into())
        }

        /// Refund all crowdfunds, which are refundable.
        ///
        /// # Arguments
        ///
        /// - `market_id`: The market id of the crowdfund.
        ///
        /// # Weight
        ///
        /// Complexity: `O(1)`
        #[frame_support::transactional]
        #[pallet::weight(5000)]
        pub fn refund(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let mut amount = <BalanceOf<T>>::zero();
            let mut removables = Vec::new();
            for ((market_id, item), backer_info) in
                <Backers<T>>::iter_prefix(&who).take(T::IterationLimit::get() as usize)
            {
                if let Some(fund_item) = <FundItems<T>>::get(&market_id, &item) {
                    match fund_item.status {
                        FundItemStatus::Refundable => {
                            removables.push((market_id, item.clone()));
                            amount = amount.saturating_add(backer_info.amount);
                        },
                        FundItemStatus::Spent => {
                            removables.push((market_id, item.clone()));
                        },
                        FundItemStatus::Active => {},
                    }
                } else {
                    log::error!(
                        "Fund item not found for market {:?} and item {:?}",
                        market_id,
                        item
                    );
                    debug_assert!(false);
                    continue;
                }
            }

            let fund_account = Self::crowdfund_account();
            T::Currency::transfer(&fund_account, &who, amount, ExistenceRequirement::AllowDeath)?;

            for (market_id, item) in removables {
                <Backers<T>>::remove(&who, (&market_id, &item));
            }

            if <Backers<T>>::iter_prefix(&who).next().is_some() {
                Self::deposit_event(Event::PartiallyRefunded { who, amount });
            } else {
                Self::deposit_event(Event::AllRefunded { who, amount });
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

    impl<T> CrowdfundPalletApi<MarketIdOf<T>, AccountIdOf<T>, BalanceOf<T>> for Pallet<T>
    where
        T: Config,
    {
        fn start_crowdfund(market_id: &MarketIdOf<T>) -> DispatchResult {
            let status = CrowdfundStatus::Active;
            let crowdfund_info = CrowdfundInfo { status };
            <Crowdfunds<T>>::insert(market_id, crowdfund_info);
            Ok(())
        }

        fn iter_items(
            market_id: &MarketIdOf<T>,
        ) -> frame_support::storage::PrefixIterator<(OutcomeReport, FundItemInfoOf<T>)> {
            <FundItems<T>>::iter_prefix(market_id)
        }

        fn set_item_status(
            market_id: &MarketIdOf<T>,
            item: &OutcomeReport,
            status: FundItemStatus,
        ) -> DispatchResult {
            let mut fund_item =
                <FundItems<T>>::get(market_id, item).ok_or(Error::<T>::FundItemNotFound)?;
            fund_item.status = status;
            <FundItems<T>>::insert(market_id, item, fund_item);
            Ok(())
        }

        fn stop_crowdfund(market_id: &MarketIdOf<T>) -> DispatchResult {
            let mut crowdfund_info =
                <Crowdfunds<T>>::get(market_id).ok_or(Error::<T>::CrowdfundNotFound)?;
            crowdfund_info.status = CrowdfundStatus::Closed;
            <Crowdfunds<T>>::insert(market_id, crowdfund_info);
            Ok(())
        }

        fn get_fund_account() -> AccountIdOf<T> {
            Self::crowdfund_account()
        }
    }
}
