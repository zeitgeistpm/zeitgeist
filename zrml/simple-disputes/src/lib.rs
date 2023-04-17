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

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
mod mock;
mod simple_disputes_pallet_api;
mod tests;
pub mod weights;

pub use pallet::*;
pub use simple_disputes_pallet_api::SimpleDisputesPalletApi;
use zeitgeist_primitives::{
    traits::{DisputeApi, DisputeResolutionApi, ZeitgeistAssetManager},
    types::{
        Asset, GlobalDisputeItem, Market, MarketDispute, MarketDisputeMechanism, MarketStatus,
        OutcomeReport, Report,
    },
};

#[frame_support::pallet]
mod pallet {
    use super::*;
    use crate::{weights::WeightInfoZeitgeist, SimpleDisputesPalletApi};
    use alloc::vec::Vec;
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::DispatchResult,
        ensure,
        pallet_prelude::{
            Blake2_128Concat, ConstU32, DispatchResultWithPostInfo, StorageMap, ValueQuery, Weight,
        },
        traits::{Currency, Get, Hooks, Imbalance, IsType, NamedReservableCurrency},
        transactional, BoundedVec, PalletId,
    };
    use frame_system::pallet_prelude::*;
    use orml_traits::currency::NamedMultiReservableCurrency;
    use sp_runtime::{
        traits::{CheckedDiv, Saturating},
        DispatchError, SaturatedConversion,
    };

    use zrml_market_commons::MarketCommonsPalletApi;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Shares of outcome assets and native currency
        type AssetManager: ZeitgeistAssetManager<
            Self::AccountId,
            Balance = <CurrencyOf<Self> as Currency<Self::AccountId>>::Balance,
            CurrencyId = Asset<MarketIdOf<Self>>,
            ReserveIdentifier = [u8; 8],
        >;

        /// Event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The base amount of currency that must be bonded in order to create a dispute.
        #[pallet::constant]
        type OutcomeBond: Get<BalanceOf<Self>>;

        type DisputeResolution: DisputeResolutionApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
            MarketId = MarketIdOf<Self>,
            Moment = MomentOf<Self>,
        >;

        /// The additional amount of currency that must be bonded when creating a subsequent
        /// dispute.
        #[pallet::constant]
        type OutcomeFactor: Get<BalanceOf<Self>>;

        /// The identifier of individual markets.
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;

        /// The maximum number of disputes allowed on any single market.
        #[pallet::constant]
        type MaxDisputes: Get<u32>;

        /// The pallet identifier.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Weights generated by benchmarks
        type WeightInfo: WeightInfoZeitgeist;
    }

    pub(crate) type BalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type CurrencyOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Currency;
    pub(crate) type NegativeImbalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;
    pub type MarketIdOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub(crate) type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;
    pub(crate) type MarketOf<T> = Market<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        <T as frame_system::Config>::BlockNumber,
        MomentOf<T>,
        Asset<MarketIdOf<T>>,
    >;
    pub(crate) type DisputesOf<T> = BoundedVec<
        MarketDispute<
            <T as frame_system::Config>::AccountId,
            <T as frame_system::Config>::BlockNumber,
            BalanceOf<T>,
        >,
        <T as Config>::MaxDisputes,
    >;
    pub type CacheSize = ConstU32<64>;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    /// For each market, this holds the dispute information for each dispute that's
    /// been issued.
    #[pallet::storage]
    pub type Disputes<T: Config> =
        StorageMap<_, Blake2_128Concat, MarketIdOf<T>, DisputesOf<T>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        OutcomeReserved {
            market_id: MarketIdOf<T>,
            dispute: MarketDispute<T::AccountId, T::BlockNumber, BalanceOf<T>>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 1. Any resolution must either have a `Disputed` or `Reported` market status
        /// 2. If status is `Disputed`, then at least one dispute must exist
        InvalidMarketStatus,
        /// On dispute or resolution, someone tried to pass a non-simple-disputes market type
        MarketDoesNotHaveSimpleDisputesMechanism,
        StorageOverflow,
        OutcomeMismatch,
        CannotDisputeSameOutcome,
        MarketIsNotReported,
        /// The maximum number of disputes has been reached.
        MaxDisputesReached,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(T::WeightInfo::suggest_outcome(
            T::MaxDisputes::get(),
            CacheSize::get(),
            CacheSize::get(),
        ))]
        #[transactional]
        pub fn suggest_outcome(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let market = T::MarketCommons::market(&market_id)?;
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::SimpleDisputes,
                Error::<T>::MarketDoesNotHaveSimpleDisputesMechanism
            );
            ensure!(market.status == MarketStatus::Disputed, Error::<T>::InvalidMarketStatus);
            ensure!(market.matches_outcome_report(&outcome), Error::<T>::OutcomeMismatch);
            let report = market.report.as_ref().ok_or(Error::<T>::MarketIsNotReported)?;

            let now = <frame_system::Pallet<T>>::block_number();
            let disputes = Disputes::<T>::get(market_id);
            let num_disputes: u32 = disputes.len().saturated_into();

            Self::ensure_can_not_dispute_the_same_outcome(&disputes, report, &outcome)?;
            Self::ensure_disputes_does_not_exceed_max_disputes(num_disputes)?;

            let bond = default_outcome_bond::<T>(disputes.len());

            T::AssetManager::reserve_named(&Self::reserve_id(), Asset::Ztg, &who, bond)?;

            let market_dispute = MarketDispute { at: now, by: who, outcome, bond };
            <Disputes<T>>::try_mutate(market_id, |disputes| {
                disputes.try_push(market_dispute.clone()).map_err(|_| <Error<T>>::StorageOverflow)
            })?;

            // each dispute resets dispute_duration
            let r = Self::remove_auto_resolve(disputes.as_slice(), &market_id, &market);
            let dispute_duration_ends_at_block =
                now.saturating_add(market.deadlines.dispute_duration);
            let e =
                T::DisputeResolution::add_auto_resolve(&market_id, dispute_duration_ends_at_block)?;

            Self::deposit_event(Event::OutcomeReserved { market_id, dispute: market_dispute });

            Ok((Some(T::WeightInfo::suggest_outcome(num_disputes, r, e))).into())
        }
    }

    impl<T: Config> Pallet<T> {
        #[inline]
        pub fn reserve_id() -> [u8; 8] {
            T::PalletId::get().0
        }

        fn ensure_can_not_dispute_the_same_outcome(
            disputes: &[MarketDispute<T::AccountId, T::BlockNumber, BalanceOf<T>>],
            report: &Report<T::AccountId, T::BlockNumber>,
            outcome: &OutcomeReport,
        ) -> DispatchResult {
            if let Some(last_dispute) = disputes.last() {
                ensure!(&last_dispute.outcome != outcome, Error::<T>::CannotDisputeSameOutcome);
            } else {
                ensure!(&report.outcome != outcome, Error::<T>::CannotDisputeSameOutcome);
            }

            Ok(())
        }

        #[inline]
        fn ensure_disputes_does_not_exceed_max_disputes(num_disputes: u32) -> DispatchResult {
            ensure!(num_disputes < T::MaxDisputes::get(), Error::<T>::MaxDisputesReached);
            Ok(())
        }

        fn get_auto_resolve(
            disputes: &[MarketDispute<T::AccountId, T::BlockNumber, BalanceOf<T>>],
            market: &MarketOf<T>,
        ) -> Option<T::BlockNumber> {
            disputes.last().map(|last_dispute| {
                last_dispute.at.saturating_add(market.deadlines.dispute_duration)
            })
        }

        fn remove_auto_resolve(
            disputes: &[MarketDispute<T::AccountId, T::BlockNumber, BalanceOf<T>>],
            market_id: &MarketIdOf<T>,
            market: &MarketOf<T>,
        ) -> u32 {
            if let Some(dispute_duration_ends_at_block) = Self::get_auto_resolve(disputes, market) {
                return T::DisputeResolution::remove_auto_resolve(
                    market_id,
                    dispute_duration_ends_at_block,
                );
            }
            0u32
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
                market.dispute_mechanism == MarketDisputeMechanism::SimpleDisputes,
                Error::<T>::MarketDoesNotHaveSimpleDisputesMechanism
            );
            Ok(())
        }

        fn on_dispute_weight() -> Weight {
            T::WeightInfo::on_dispute_weight()
        }

        fn on_resolution(
            market_id: &Self::MarketId,
            market: &MarketOf<T>,
        ) -> Result<Option<OutcomeReport>, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::SimpleDisputes,
                Error::<T>::MarketDoesNotHaveSimpleDisputesMechanism
            );
            ensure!(market.status == MarketStatus::Disputed, Error::<T>::InvalidMarketStatus);

            let disputes = Disputes::<T>::get(market_id);

            let last_dispute = match disputes.last() {
                Some(l) => l,
                // if there are no disputes, then the market is resolved with the default report
                None => return Ok(None),
            };

            Ok(Some(last_dispute.outcome.clone()))
        }

        fn exchange(
            market_id: &Self::MarketId,
            market: &MarketOf<T>,
            resolved_outcome: &OutcomeReport,
            mut overall_imbalance: NegativeImbalanceOf<T>,
        ) -> Result<NegativeImbalanceOf<T>, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::SimpleDisputes,
                Error::<T>::MarketDoesNotHaveSimpleDisputesMechanism
            );
            ensure!(market.status == MarketStatus::Disputed, Error::<T>::InvalidMarketStatus);

            let disputes = Disputes::<T>::get(market_id);

            let mut correct_reporters: Vec<T::AccountId> = Vec::new();

            for dispute in disputes.iter() {
                if &dispute.outcome == resolved_outcome {
                    T::AssetManager::unreserve_named(
                        &Self::reserve_id(),
                        Asset::Ztg,
                        &dispute.by,
                        dispute.bond.saturated_into::<u128>().saturated_into(),
                    );

                    correct_reporters.push(dispute.by.clone());
                } else {
                    let (imbalance, _) = CurrencyOf::<T>::slash_reserved_named(
                        &Self::reserve_id(),
                        &dispute.by,
                        dispute.bond.saturated_into::<u128>().saturated_into(),
                    );
                    overall_imbalance.subsume(imbalance);
                }
            }

            // Fold all the imbalances into one and reward the correct reporters. The
            // number of correct reporters might be zero if the market defaults to the
            // report after abandoned dispute. In that case, the rewards remain slashed.
            if let Some(reward_per_each) =
                overall_imbalance.peek().checked_div(&correct_reporters.len().saturated_into())
            {
                for correct_reporter in &correct_reporters {
                    let (actual_reward, leftover) = overall_imbalance.split(reward_per_each);
                    overall_imbalance = leftover;
                    CurrencyOf::<T>::resolve_creating(correct_reporter, actual_reward);
                }
            }

            Disputes::<T>::remove(market_id);

            Ok(overall_imbalance)
        }

        fn get_auto_resolve(
            market_id: &Self::MarketId,
            market: &MarketOf<T>,
        ) -> Result<Option<Self::BlockNumber>, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::SimpleDisputes,
                Error::<T>::MarketDoesNotHaveSimpleDisputesMechanism
            );
            let disputes = Disputes::<T>::get(market_id);
            Ok(Self::get_auto_resolve(disputes.as_slice(), market))
        }

        fn has_failed(
            market_id: &Self::MarketId,
            market: &MarketOf<T>,
        ) -> Result<bool, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::SimpleDisputes,
                Error::<T>::MarketDoesNotHaveSimpleDisputesMechanism
            );
            let disputes = <Disputes<T>>::get(market_id);
            Ok(disputes.len() == T::MaxDisputes::get() as usize)
        }

        fn on_global_dispute(
            market_id: &Self::MarketId,
            market: &MarketOf<T>,
        ) -> Result<Vec<GlobalDisputeItem<Self::AccountId, Self::Balance>>, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::SimpleDisputes,
                Error::<T>::MarketDoesNotHaveSimpleDisputesMechanism
            );

            Ok(<Disputes<T>>::get(market_id)
                .iter()
                .map(|dispute| GlobalDisputeItem {
                    outcome: dispute.outcome.clone(),
                    owner: dispute.by.clone(),
                    initial_vote_amount: dispute.bond,
                })
                .collect())
        }

        fn clear(market_id: &Self::MarketId, market: &MarketOf<T>) -> DispatchResult {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::SimpleDisputes,
                Error::<T>::MarketDoesNotHaveSimpleDisputesMechanism
            );
            // `Disputes` is emtpy unless the market is disputed, so this is just a defensive
            // check.
            if market.status == MarketStatus::Disputed {
                for dispute in Disputes::<T>::take(market_id).iter() {
                    T::AssetManager::unreserve_named(
                        &Self::reserve_id(),
                        Asset::Ztg,
                        &dispute.by,
                        dispute.bond.saturated_into::<u128>().saturated_into(),
                    );
                }
            }
            Ok(())
        }
    }

    impl<T> SimpleDisputesPalletApi for Pallet<T> where T: Config {}

    // No-one can bound more than BalanceOf<T>, therefore, this functions saturates
    pub fn default_outcome_bond<T>(n: usize) -> BalanceOf<T>
    where
        T: Config,
    {
        T::OutcomeBond::get().saturating_add(
            T::OutcomeFactor::get().saturating_mul(n.saturated_into::<u32>().into()),
        )
    }
}

#[cfg(any(feature = "runtime-benchmarks", test))]
pub(crate) fn market_mock<T>() -> MarketOf<T>
where
    T: crate::Config,
{
    use frame_support::traits::Get;
    use sp_runtime::{traits::AccountIdConversion, SaturatedConversion};
    use zeitgeist_primitives::types::{MarketBonds, ScoringRule};

    zeitgeist_primitives::types::Market {
        base_asset: Asset::Ztg,
        creation: zeitgeist_primitives::types::MarketCreation::Permissionless,
        creator_fee: 0,
        creator: T::PalletId::get().into_account_truncating(),
        market_type: zeitgeist_primitives::types::MarketType::Scalar(0..=100),
        dispute_mechanism: zeitgeist_primitives::types::MarketDisputeMechanism::SimpleDisputes,
        metadata: Default::default(),
        oracle: T::PalletId::get().into_account_truncating(),
        period: zeitgeist_primitives::types::MarketPeriod::Block(Default::default()),
        deadlines: zeitgeist_primitives::types::Deadlines {
            grace_period: 1_u32.into(),
            oracle_duration: 1_u32.into(),
            dispute_duration: 42_u32.into(),
        },
        report: Some(zeitgeist_primitives::types::Report {
            outcome: OutcomeReport::Scalar(0),
            at: 0u64.saturated_into(),
            by: T::PalletId::get().into_account_truncating(),
        }),
        resolved_outcome: None,
        scoring_rule: ScoringRule::CPMM,
        status: zeitgeist_primitives::types::MarketStatus::Disputed,
        bonds: MarketBonds::default(),
    }
}
