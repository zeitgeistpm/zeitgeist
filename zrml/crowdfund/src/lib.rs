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
            DispatchResultWithPostInfo, OptionQuery, StorageDoubleMap, StorageMap, ValueQuery,
        },
        sp_runtime::traits::StaticLookup,
        traits::{
            Currency, ExistenceRequirement, Get, IsType, LockIdentifier, ReservableCurrency,
            WithdrawReasons,
        },
        Blake2_128Concat, BoundedVec, PalletId, Twox64Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use sp_runtime::{
        traits::{AccountIdConversion, CheckedDiv, Saturating, Zero},
        DispatchResult,
    };
    use sp_std::{vec, vec::Vec};
    use zeitgeist_primitives::types::OutcomeReport;
    use zrml_market_commons::MarketCommonsPalletApi;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The currency implementation used to lock tokens for voting.
        type Currency: ReservableCurrency<Self::AccountId>;

        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The pallet identifier.
        #[pallet::constant]
        type CrowdfundPalletId: Get<PalletId>;

        /// To reference the market id type.
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;

        /// The unique item type which is used to identify fund items.
        type UniqueItem: Parameter
            + Member
            + Copy
            + MaybeSerializeDeserialize
            + Default
            + TypeInfo
            + Ord
            + Bounded
            + AtLeast32BitUnsigned
            + MaxEncodedLen
            + CheckedDiv
            + Zero;
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
        T::UnqiueItem,
        FundItemInfoOf<T>,
        OptionQuery,
    >;

    #[pallet::storage]
    pub type Backers<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        AccountIdOf<T>,
        Blake2_128Concat,
        (MarketIdOf<T>, T::UniqueItem),
        BackerInfoOf<T>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {}

    #[pallet::error]
    pub enum Error<T> {}

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
            market_id: AccountIdLookupOf<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;

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
        fn start_crowdfund(
            market_id: &MarketIdOf<T>,
        ) -> DispatchResult {
            Ok(())
        }

        fn stop_crowdfund(
            market_id: &MarketIdOf<T>,
        ) -> DispatchResult {
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
