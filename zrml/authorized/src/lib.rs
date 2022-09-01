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

//! # Authorized

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod authorized_pallet_api;
mod benchmarks;
pub mod migrations;
mod mock;
mod tests;
pub mod weights;

pub use authorized_pallet_api::AuthorizedPalletApi;
pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::{weights::WeightInfoZeitgeist, AuthorizedPalletApi};
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::DispatchResult,
        ensure,
        pallet_prelude::{EnsureOrigin, OptionQuery, StorageMap},
        traits::{Currency, Get, Hooks, IsType, StorageVersion},
        PalletId, Twox64Concat,
    };
    use frame_system::pallet_prelude::OriginFor;
    use sp_runtime::DispatchError;
    use zeitgeist_primitives::{
        traits::DisputeApi,
        types::{Market, MarketDispute, MarketDisputeMechanism, MarketStatus, OutcomeReport},
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    pub(crate) type BalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type CurrencyOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Currency;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub(crate) type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Overwrites already provided outcomes for the same market and account.
        #[frame_support::transactional]
        #[pallet::weight(T::WeightInfo::authorize_market_outcome())]
        pub fn authorize_market_outcome(
            origin: OriginFor<T>,
            market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
        ) -> DispatchResult {
            T::AuthorizedDisputeResolutionOrigin::ensure_origin(origin)
                .map_err(|_| Error::<T>::NotAuthorizedForDisputeResolution)?;
            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.status == MarketStatus::Disputed, Error::<T>::MarketIsNotDisputed);
            ensure!(market.matches_outcome_report(&outcome), Error::<T>::OutcomeMismatch);
            match market.dispute_mechanism {
                MarketDisputeMechanism::Authorized => {
                    AuthorizedOutcomeReports::<T>::insert(market_id, outcome);
                    Ok(())
                }
                _ => {
                    Err(Error::<T>::MarketDoesNotHaveDisputeMechanismAuthorized.into())
                }
            }
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Market commons
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;

        /// The origin that is allowed to resolved disupute in Authorized dispute mechanism.
        type AuthorizedDisputeResolutionOrigin: EnsureOrigin<Self::Origin>;

        /// Identifier of this pallet
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Weights generated by benchmarks
        type WeightInfo: WeightInfoZeitgeist;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// An unauthorized account attempts to submit a report.
        NotAuthorizedForDisputeResolution,
        /// The market unexpectedly has the incorrect dispute mechanism.
        MarketDoesNotHaveDisputeMechanismAuthorized,
        /// An account attempts to submit a report to an undisputed market.
        MarketIsNotDisputed,
        /// The report does not match the market's type.
        OutcomeMismatch,
    }

    #[pallet::event]
    pub enum Event<T>
    where
        T: Config, {}

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    impl<T> Pallet<T> where T: Config {}

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
            _: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            _: &Self::MarketId,
            _: &Market<Self::AccountId, Self::BlockNumber, Self::Moment>,
        ) -> DispatchResult {
            Ok(())
        }

        fn on_resolution(
            _: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            market_id: &Self::MarketId,
            _: &Market<Self::AccountId, Self::BlockNumber, MomentOf<T>>,
        ) -> Result<Option<OutcomeReport>, DispatchError> {
            let result = AuthorizedOutcomeReports::<T>::get(market_id);
            if result.is_some() {
                AuthorizedOutcomeReports::<T>::remove(market_id);
            }
            Ok(result)
        }
    }

    impl<T> AuthorizedPalletApi for Pallet<T> where T: Config {}

    /// Maps the market id to the outcome reported by the authorized account.    
    #[pallet::storage]
    #[pallet::getter(fn outcomes)]
    pub type AuthorizedOutcomeReports<T: Config> =
        StorageMap<_, Twox64Concat, MarketIdOf<T>, OutcomeReport, OptionQuery>;
}

#[cfg(any(feature = "runtime-benchmarks", test))]
pub(crate) fn market_mock<T>()
-> zeitgeist_primitives::types::Market<T::AccountId, T::BlockNumber, MomentOf<T>>
where
    T: crate::Config,
{
    use frame_support::traits::Get;
    use sp_runtime::traits::AccountIdConversion;
    use zeitgeist_primitives::types::ScoringRule;

    zeitgeist_primitives::types::Market {
        creation: zeitgeist_primitives::types::MarketCreation::Permissionless,
        creator_fee: 0,
        creator: T::PalletId::get().into_account(),
        market_type: zeitgeist_primitives::types::MarketType::Scalar(0..=100),
        dispute_mechanism: zeitgeist_primitives::types::MarketDisputeMechanism::Authorized,
        metadata: Default::default(),
        oracle: T::PalletId::get().into_account(),
        period: zeitgeist_primitives::types::MarketPeriod::Block(Default::default()),
        report: None,
        resolved_outcome: None,
        scoring_rule: ScoringRule::CPMM,
        status: zeitgeist_primitives::types::MarketStatus::Disputed,
    }
}
