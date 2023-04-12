// Copyright 2022-2023 Forecasting Technologies LTD.
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
mod mock_storage;
mod tests;
pub mod weights;

pub use authorized_pallet_api::AuthorizedPalletApi;
pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::{weights::WeightInfoZeitgeist, AuthorizedPalletApi};
    use alloc::vec::Vec;
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::{DispatchResult, DispatchResultWithPostInfo},
        ensure,
        pallet_prelude::{ConstU32, EnsureOrigin, OptionQuery, StorageMap},
        traits::{Currency, Get, Hooks, IsType, StorageVersion},
        PalletId, Twox64Concat,
    };
    use frame_system::pallet_prelude::OriginFor;
    use sp_runtime::{traits::Saturating, DispatchError};
    use zeitgeist_primitives::{
        traits::{DisputeApi, DisputeResolutionApi},
        types::{
            Asset, AuthorityReport, GlobalDisputeItem, Market, MarketDisputeMechanism,
            MarketStatus, OutcomeReport,
        },
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(3);

    pub(crate) type BalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type CurrencyOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Currency;
    pub(crate) type NegativeImbalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub(crate) type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;
    pub type CacheSize = ConstU32<64>;
    pub(crate) type MarketOf<T> = Market<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        <T as frame_system::Config>::BlockNumber,
        MomentOf<T>,
        Asset<MarketIdOf<T>>,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Overwrites already provided outcomes for the same market and account.
        #[frame_support::transactional]
        #[pallet::weight(
            T::WeightInfo::authorize_market_outcome_first_report(CacheSize::get()).max(
                T::WeightInfo::authorize_market_outcome_existing_report(),
            )
        )]
        pub fn authorize_market_outcome(
            origin: OriginFor<T>,
            market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
        ) -> DispatchResultWithPostInfo {
            T::AuthorizedDisputeResolutionOrigin::ensure_origin(origin)?;
            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.status == MarketStatus::Disputed, Error::<T>::MarketIsNotDisputed);
            ensure!(market.matches_outcome_report(&outcome), Error::<T>::OutcomeMismatch);
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Authorized,
                Error::<T>::MarketDoesNotHaveDisputeMechanismAuthorized
            );

            let now = frame_system::Pallet::<T>::block_number();

            let report_opt = AuthorizedOutcomeReports::<T>::get(market_id);
            let (report, ids_len) = match &report_opt {
                Some(report) => (
                    AuthorityReport { resolve_at: report.resolve_at, outcome: outcome.clone() },
                    0u32,
                ),
                None => {
                    let resolve_at = now.saturating_add(T::CorrectionPeriod::get());
                    let ids_len = T::DisputeResolution::add_auto_resolve(&market_id, resolve_at)?;
                    (AuthorityReport { resolve_at, outcome: outcome.clone() }, ids_len)
                }
            };

            AuthorizedOutcomeReports::<T>::insert(market_id, report);

            Self::deposit_event(Event::AuthorityReported { market_id, outcome });

            if report_opt.is_none() {
                Ok(Some(T::WeightInfo::authorize_market_outcome_first_report(ids_len)).into())
            } else {
                Ok(Some(T::WeightInfo::authorize_market_outcome_existing_report()).into())
            }
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The period, in which the authority can correct the outcome of a market.
        /// This value must not be zero.
        #[pallet::constant]
        type CorrectionPeriod: Get<Self::BlockNumber>;

        type DisputeResolution: DisputeResolutionApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
            MarketId = MarketIdOf<Self>,
            Moment = MomentOf<Self>,
        >;

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
        /// The market unexpectedly has the incorrect dispute mechanism.
        MarketDoesNotHaveDisputeMechanismAuthorized,
        /// An account attempts to submit a report to an undisputed market.
        MarketIsNotDisputed,
        /// The report does not match the market's type.
        OutcomeMismatch,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// The Authority reported.
        AuthorityReported { market_id: MarketIdOf<T>, outcome: OutcomeReport },
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    impl<T> Pallet<T>
    where
        T: Config,
    {
        /// Return the resolution block number for the given market.
        fn get_auto_resolve(market_id: &MarketIdOf<T>) -> Option<T::BlockNumber> {
            AuthorizedOutcomeReports::<T>::get(market_id).map(|report| report.resolve_at)
        }
    }

    impl<T> DisputeApi for Pallet<T>
    where
        T: Config,
    {
        type AccountId = T::AccountId;
        type Balance = BalanceOf<T>;
        type NegativeImbalance = NegativeImbalanceOf<T>;
        type BlockNumber = T::BlockNumber;
        type MarketId = MarketIdOf<T>;
        type Moment = MomentOf<T>;
        type Origin = T::Origin;

        fn on_dispute(_: &Self::MarketId, market: &MarketOf<T>) -> DispatchResult {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Authorized,
                Error::<T>::MarketDoesNotHaveDisputeMechanismAuthorized
            );
            Ok(())
        }

        fn on_resolution(
            market_id: &Self::MarketId,
            market: &MarketOf<T>,
        ) -> Result<Option<OutcomeReport>, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Authorized,
                Error::<T>::MarketDoesNotHaveDisputeMechanismAuthorized
            );
            let report = AuthorizedOutcomeReports::<T>::take(market_id);
            Ok(report.map(|r| r.outcome))
        }

        fn exchange(
            _: &Self::MarketId,
            market: &MarketOf<T>,
            _: &OutcomeReport,
            overall_imbalance: NegativeImbalanceOf<T>,
        ) -> Result<NegativeImbalanceOf<T>, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Authorized,
                Error::<T>::MarketDoesNotHaveDisputeMechanismAuthorized
            );
            // all funds to treasury
            Ok(overall_imbalance)
        }

        fn get_auto_resolve(
            market_id: &Self::MarketId,
            market: &MarketOf<T>,
        ) -> Result<Option<Self::BlockNumber>, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Authorized,
                Error::<T>::MarketDoesNotHaveDisputeMechanismAuthorized
            );
            Ok(Self::get_auto_resolve(market_id))
        }

        fn has_failed(_: &Self::MarketId, market: &MarketOf<T>) -> Result<bool, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Authorized,
                Error::<T>::MarketDoesNotHaveDisputeMechanismAuthorized
            );

            Ok(false)
        }

        fn on_global_dispute(
            _: &Self::MarketId,
            market: &MarketOf<T>,
        ) -> Result<Vec<GlobalDisputeItem<Self::AccountId, Self::Balance>>, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Authorized,
                Error::<T>::MarketDoesNotHaveDisputeMechanismAuthorized
            );

            Ok(Vec::new())
        }

        fn clear(market_id: &Self::MarketId, market: &MarketOf<T>) -> DispatchResult {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Authorized,
                Error::<T>::MarketDoesNotHaveDisputeMechanismAuthorized
            );

            AuthorizedOutcomeReports::<T>::remove(market_id);
            Ok(())
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
pub(crate) fn market_mock<T>() -> MarketOf<T>
where
    T: crate::Config,
{
    use frame_support::traits::Get;
    use sp_runtime::traits::AccountIdConversion;
    use zeitgeist_primitives::types::{Asset, MarketBonds, ScoringRule};

    zeitgeist_primitives::types::Market {
        base_asset: Asset::Ztg,
        creation: zeitgeist_primitives::types::MarketCreation::Permissionless,
        creator_fee: 0,
        creator: T::PalletId::get().into_account_truncating(),
        market_type: zeitgeist_primitives::types::MarketType::Scalar(0..=100),
        dispute_mechanism: zeitgeist_primitives::types::MarketDisputeMechanism::Authorized,
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
        bonds: MarketBonds::default(),
    }
}
