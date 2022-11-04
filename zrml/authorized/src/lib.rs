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
        pallet_prelude::{OptionQuery, StorageMap},
        traits::{Currency, Get, Hooks, IsType, StorageVersion},
        PalletId, Twox64Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use sp_runtime::{traits::Saturating, DispatchError};
    use zeitgeist_primitives::{
        traits::{DisputeApi, DisputeResolutionApi},
        types::{
            AuthorityReport, Market, MarketDispute, MarketDisputeMechanism, MarketStatus,
            OutcomeReport,
        },
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

    pub(crate) type BalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type CurrencyOf<T> =
        <<T as Config>::MarketCommonsAuthorized as MarketCommonsPalletApi>::Currency;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommonsAuthorized as MarketCommonsPalletApi>::MarketId;
    pub(crate) type MomentOf<T> =
        <<T as Config>::MarketCommonsAuthorized as MarketCommonsPalletApi>::Moment;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // TODO update benchmark
        /// Overwrites already provided outcomes for the same market and account.
        #[frame_support::transactional]
        #[pallet::weight(T::WeightInfo::authorize_market_outcome())]
        pub fn authorize_market_outcome(
            origin: OriginFor<T>,
            market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let market = T::MarketCommonsAuthorized::market(&market_id)?;
            ensure!(market.status == MarketStatus::Disputed, Error::<T>::MarketIsNotDisputed);
            ensure!(market.matches_outcome_report(&outcome), Error::<T>::OutcomeMismatch);
            if let MarketDisputeMechanism::Authorized(ref account_id) = market.dispute_mechanism {
                if account_id != &who {
                    return Err(Error::<T>::NotAuthorizedForThisMarket.into());
                }
            } else {
                return Err(Error::<T>::MarketDoesNotHaveDisputeMechanismAuthorized.into());
            }

            Self::remove_auto_resolve(&market_id);
            let now = frame_system::Pallet::<T>::block_number();
            let correction_period_ends_at = now.saturating_add(T::CorrectionPeriod::get());
            T::DisputeResolution::add_auto_resolve(&market_id, correction_period_ends_at)?;

            let report = AuthorityReport { resolve_at: correction_period_ends_at, outcome };
            AuthorizedOutcomeReports::<T>::insert(market_id, report);

            Ok(())
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The period in which the authority has to report.
        #[pallet::constant]
        type AuthorityReportPeriod: Get<Self::BlockNumber>;

        /// Event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The period, in which the authority can correct the outcome of a market.
        #[pallet::constant]
        type CorrectionPeriod: Get<Self::BlockNumber>;

        type DisputeResolution: DisputeResolutionApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
            MarketId = MarketIdOf<Self>,
            Moment = MomentOf<Self>,
        >;

        /// Market commons
        type MarketCommonsAuthorized: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;

        /// Identifier of this pallet
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Weights generated by benchmarks
        type WeightInfo: WeightInfoZeitgeist;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The authority already made its report.
        AuthorityAlreadyReported,
        /// An unauthorized account attempts to submit a report.
        NotAuthorizedForThisMarket,
        /// The market unexpectedly has the incorrect dispute mechanism.
        MarketDoesNotHaveDisputeMechanismAuthorized,
        /// An account attempts to submit a report to an undisputed market.
        MarketIsNotDisputed,
        /// The report does not match the market's type.
        OutcomeMismatch,
        /// The market should be reported at this point.
        MarketIsNotReported,
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

    impl<T> Pallet<T>
    where
        T: Config,
    {
        fn get_auto_resolve(market_id: &MarketIdOf<T>) -> Option<T::BlockNumber> {
            AuthorizedOutcomeReports::<T>::get(market_id).map(|report| report.resolve_at)
        }

        fn remove_auto_resolve(market_id: &MarketIdOf<T>) {
            if let Some(resolve_at) = Self::get_auto_resolve(market_id) {
                T::DisputeResolution::remove_auto_resolve(market_id, resolve_at);
            }
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
            _: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            market_id: &Self::MarketId,
            market: &Market<Self::AccountId, Self::BlockNumber, Self::Moment>,
        ) -> DispatchResult {
            if let MarketDisputeMechanism::Authorized(_) = market.dispute_mechanism {
                if AuthorizedOutcomeReports::<T>::get(market_id).is_some() {
                    return Err(Error::<T>::AuthorityAlreadyReported.into());
                }
                Ok(())
            } else {
                Err(Error::<T>::MarketDoesNotHaveDisputeMechanismAuthorized.into())
            }
        }

        fn on_resolution(
            _: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            market_id: &Self::MarketId,
            market: &Market<Self::AccountId, Self::BlockNumber, MomentOf<T>>,
        ) -> Result<Option<OutcomeReport>, DispatchError> {
            if let MarketDisputeMechanism::Authorized(_) = market.dispute_mechanism {
                let result = AuthorizedOutcomeReports::<T>::get(market_id);
                if result.is_some() {
                    AuthorizedOutcomeReports::<T>::remove(market_id);
                }
                Ok(result.map(|report| report.outcome))
            } else {
                Err(Error::<T>::MarketDoesNotHaveDisputeMechanismAuthorized.into())
            }
        }

        fn get_auto_resolve(
            _: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            market_id: &Self::MarketId,
            market: &Market<Self::AccountId, Self::BlockNumber, MomentOf<T>>,
        ) -> Result<Option<Self::BlockNumber>, DispatchError> {
            if let MarketDisputeMechanism::Authorized(_) = market.dispute_mechanism {
                Ok(Self::get_auto_resolve(market_id))
            } else {
                Err(Error::<T>::MarketDoesNotHaveDisputeMechanismAuthorized.into())
            }
        }

        fn is_fail(
            _: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            market_id: &Self::MarketId,
            market: &Market<Self::AccountId, Self::BlockNumber, MomentOf<T>>,
        ) -> Result<bool, DispatchError> {
            if let MarketDisputeMechanism::Authorized(_) = market.dispute_mechanism {
                let is_unreported = !AuthorizedOutcomeReports::<T>::contains_key(market_id);
                let report = market.report.as_ref().ok_or(Error::<T>::MarketIsNotReported)?;
                let now = frame_system::Pallet::<T>::block_number();
                let is_expired = report.at.saturating_add(T::AuthorityReportPeriod::get()) < now;
                Ok(is_unreported && is_expired)
            } else {
                Err(Error::<T>::MarketDoesNotHaveDisputeMechanismAuthorized.into())
            }
        }
    }

    impl<T> AuthorizedPalletApi for Pallet<T> where T: Config {}

    /// Maps the market id to the outcome reported by the authorized account.    
    #[pallet::storage]
    #[pallet::getter(fn outcomes)]
    pub type AuthorizedOutcomeReports<T: Config> =
        StorageMap<_, Twox64Concat, MarketIdOf<T>, AuthorityReport<T::BlockNumber>, OptionQuery>;
}

#[cfg(any(feature = "runtime-benchmarks", test))]
pub(crate) fn market_mock<T>(
    ai: T::AccountId,
) -> zeitgeist_primitives::types::Market<T::AccountId, T::BlockNumber, MomentOf<T>>
where
    T: crate::Config,
{
    use frame_support::traits::Get;
    use sp_runtime::traits::AccountIdConversion;
    use zeitgeist_primitives::types::ScoringRule;

    zeitgeist_primitives::types::Market {
        creation: zeitgeist_primitives::types::MarketCreation::Permissionless,
        creator_fee: 0,
        creator: T::PalletId::get().into_account_truncating(),
        market_type: zeitgeist_primitives::types::MarketType::Scalar(0..=100),
        dispute_mechanism: zeitgeist_primitives::types::MarketDisputeMechanism::Authorized(ai),
        metadata: Default::default(),
        oracle: T::PalletId::get().into_account_truncating(),
        period: zeitgeist_primitives::types::MarketPeriod::Block(Default::default()),
        deadlines: zeitgeist_primitives::types::Deadlines {
            grace_period: 1_u32.into(),
            oracle_duration: 1_u32.into(),
            dispute_duration: 1_u32.into(),
        },
        report: None,
        resolved_outcome: None,
        scoring_rule: ScoringRule::CPMM,
        status: zeitgeist_primitives::types::MarketStatus::Disputed,
    }
}
