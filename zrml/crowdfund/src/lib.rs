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
        pallet_prelude::{
            DispatchResultWithPostInfo,
            OptionQuery, StorageDoubleMap, StorageMap, ValueQuery,
        },
        traits::{
            Currency, Get, IsType, ReservableCurrency,
        },
        Blake2_128Concat, BoundedVec, PalletId, Parameter, Twox64Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use sp_runtime::{
        traits::{
            AccountIdConversion, AtLeast32BitUnsigned, Bounded, CheckedDiv, Saturating, Zero,
        },
        DispatchResult,
    };
    use sp_std::{vec, vec::Vec};
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
        type Currency: ReservableCurrency<Self::AccountId>;

        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// To reference the market id type.
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;

        #[pallet::constant]
        type MinFunding: Get<BalanceOf<Self>>;
    }

    pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

    pub type CrowdfundInfoOf<T> = CrowdfundInfo<BalanceOf<T>>;
    pub type FundItemInfoOf<T> = FundItemInfo<BalanceOf<T>>;
    pub type BackerInfoOf<T> = BackerInfo<BalanceOf<T>>;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::storage]
    pub type Crowdfunds<T: Config> =
        StorageMap<_, Twox64Concat, MarketIdOf<T>, CrowdfundInfoOf<T>, OptionQuery>;

    #[pallet::storage]
    pub type FundItems<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        MarketIdOf<T>,
        Blake2_128Concat,
        UniqueFundItem,
        FundItemInfoOf<T>,
        OptionQuery,
    >;

    #[pallet::storage]
    pub type Backers<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        AccountIdOf<T>,
        Blake2_128Concat,
        (MarketIdOf<T>, UniqueFundItem),
        BackerInfoOf<T>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config, {}

    #[pallet::error]
    pub enum Error<T> {
        CrowdfundNotFound,
        NoOutcomeWinner,
        AmountTooLow,
        CrowdfundNotActive,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Make a crowdfund.
        ///
        /// # Arguments
        ///
        /// - `voter`: The account id lookup to unlock funds for.
        ///
        /// # Weight
        ///
        /// Complexity: `O(1)`
        #[frame_support::transactional]
        #[pallet::weight(5000)]
        pub fn fund(
            origin: OriginFor<T>,
            market_id: MarketIdOf<T>,
            item: UniqueFundItem,
            amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            ensure!(amount > T::MinFunding::get(), Error::<T>::AmountTooLow);
            let crowdfund_info = <Crowdfunds<T>>::get(&market_id).ok_or(Error::<T>::CrowdfundNotFound)?;
            ensure!(crowdfund_info.status == CrowdfundStatus::Active, Error::<T>::CrowdfundNotActive);

            match &item {
                UniqueFundItem::Outcome(outcome) => {},
                UniqueFundItem::Appeal(appeal_number) => {},
            }

            // TODO check if minimum is reached
            let mut fund_item = <FundItems<T>>::get(&market_id, &item)
                .unwrap_or(FundItemInfo { raised: Zero::zero() });
            fund_item.raised = fund_item.raised.saturating_add(amount);

            let mut backer = <Backers<T>>::get(&who, (&market_id, &item))
                .unwrap_or(BackerInfo { reserved: Zero::zero() });
            backer.reserved = backer.reserved.saturating_add(amount);

            <FundItems<T>>::insert(&market_id, &item, fund_item);

            <Backers<T>>::insert(&who, &(market_id, item), backer);

            Ok(Some(5000).into())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn crowdfund_account(market_id: &MarketIdOf<T>) -> T::AccountId {
            T::CrowdfundPalletId::get().into_sub_account_truncating(market_id)
        }
    }

    impl<T> CrowdfundPalletApi<MarketIdOf<T>, AccountIdOf<T>, BalanceOf<T>> for Pallet<T>
    where
        T: Config,
    {
        fn start_crowdfund(market_id: &MarketIdOf<T>) -> DispatchResult {
            let status = CrowdfundStatus::Active;
            let appeal_threshold = T::AppealThreshold::get();
            let crowdfund_info = CrowdfundInfo { status, appeal_threshold };
            <Crowdfunds<T>>::insert(market_id, crowdfund_info);
            Ok(())
        }

        fn stop_crowdfund(market_id: &MarketIdOf<T>, winner: UniqueFundItem) -> DispatchResult {
            let mut crowdfund_info =
                <Crowdfunds<T>>::get(market_id).ok_or(Error::<T>::CrowdfundNotFound)?;
            let winner_outcome = match winner {
                UniqueFundItem::Outcome(outcome) => outcome,
                _ => return Err(Error::<T>::NoOutcomeWinner.into()),
            };
            crowdfund_info.status = CrowdfundStatus::Finished { winner: winner_outcome };
            <Crowdfunds<T>>::insert(market_id, crowdfund_info);
            Ok(())
        }

        fn get_looser_stake(market_id: &MarketIdOf<T>) -> BalanceOf<T> {
            <BalanceOf<T>>::zero()
        }

        fn get_party_account(market_id: &MarketIdOf<T>) -> AccountIdOf<T> {
            Self::crowdfund_account(market_id)
        }
    }
}
