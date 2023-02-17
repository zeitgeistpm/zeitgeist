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

// It is important to note that if a categorical market has only two outcomes, then winners
// won't receive any rewards because accounts of the most voted outcome on the loser side are
// simply registered as `JurorStatus::Tardy`.

#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::type_complexity)]

extern crate alloc;

mod benchmarks;
mod court_pallet_api;
pub mod migrations;
mod mock;
mod tests;
mod types;
pub mod weights;

pub use court_pallet_api::CourtPalletApi;
pub use pallet::*;
pub use types::*;

// TODO: remove this crowdfund interface and use the real after crowdfund pallet is merged
use frame_support::pallet_prelude::{DispatchError, DispatchResult};
use zeitgeist_primitives::types::OutcomeReport;

/// The trait for handling of crowdfunds.
pub trait CrowdfundPalletApi<AccountId, Balance, NegativeImbalance> {
    /// Create a new crowdfund.
    ///
    /// # Returns
    /// - `FundIndex` - The id of the crowdfund.
    fn open_crowdfund() -> Result<u128, DispatchError>;

    /// Get an iterator over all items of a crowdfund.
    ///
    /// # Arguments
    /// - `fund_index` - The id of the crowdfund.
    ///
    /// # Returns
    /// - `PrefixIterator` - The iterator over all items of the crowdfund.
    fn iter_items(
        fund_index: u128,
    ) -> frame_support::storage::PrefixIterator<(OutcomeReport, Balance)>;

    /// Maybe get an item of a crowdfund.
    ///
    /// # Arguments
    /// - `fund_index` - The id of the crowdfund.
    /// - `item` - The item to get.
    ///
    /// # Returns
    /// - `Option<Balance>` - The balance of the item.
    fn get_item(fund_index: u128, item: &OutcomeReport) -> Option<Balance>;

    /// Prepare for all related backers to potentially refund their stake.
    ///
    /// # Arguments
    /// - `fund_index` - The id of the crowdfund.
    /// - `item` - The item to refund.
    /// - `fee` - The overall fee to charge from the fund item
    ///  before the backer refunds are possible.
    ///
    /// # Returns
    /// - `NegativeImbalance` - The imbalance that contains the charged fees.
    fn prepare_refund(
        fund_index: u128,
        item: &OutcomeReport,
        fee: sp_runtime::Percent,
    ) -> Result<NegativeImbalance, DispatchError>;

    /// Close a crowdfund.
    ///
    /// # Arguments
    /// - `fund_index` - The id of the crowdfund.
    fn close_crowdfund(fund_index: u128) -> DispatchResult;
}

#[frame_support::pallet]
mod pallet {
    use crate::{
        weights::WeightInfoZeitgeist, CourtInfo, CourtPalletApi, CrowdfundInfo, CrowdfundPalletApi,
        JurorInfo, Periods, Vote,
    };
    use alloc::{
        collections::{BTreeMap, BTreeSet},
        vec::Vec,
    };
    use arrayvec::ArrayVec;
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::DispatchResult,
        ensure, log,
        pallet_prelude::{OptionQuery, StorageDoubleMap, StorageMap, StorageValue, ValueQuery},
        traits::{
            BalanceStatus, Currency, Get, IsType, NamedReservableCurrency, Randomness,
            StorageVersion,
        },
        transactional, Blake2_128Concat, BoundedVec, PalletId,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use rand::{rngs::StdRng, seq::SliceRandom, RngCore, SeedableRng};
    use sp_runtime::{
        traits::{AccountIdConversion, CheckedDiv, Hash, Saturating},
        DispatchError, Percent,
    };
    use zeitgeist_primitives::{
        traits::{DisputeApi, DisputeResolutionApi},
        types::{Asset, Market, MarketDisputeMechanism, MarketStatus, OutcomeReport},
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[pallet::constant]
        type CourtCrowdfundPeriod: Get<Self::BlockNumber>;

        #[pallet::constant]
        type CourtVotePeriod: Get<Self::BlockNumber>;

        #[pallet::constant]
        type CourtAggregationPeriod: Get<Self::BlockNumber>;

        #[pallet::constant]
        type CourtAppealPeriod: Get<Self::BlockNumber>;

        type Crowdfund: crate::CrowdfundPalletApi<
            Self::AccountId,
            BalanceOf<Self>,
            NegativeImbalanceOf<Self>,
        >;

        #[pallet::constant]
        type CrowdfundMinThreshold: Get<BalanceOf<Self>>;

        type DisputeResolution: DisputeResolutionApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
            MarketId = MarketIdOf<Self>,
            Moment = MomentOf<Self>,
        >;

        /// Event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Market commons
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;

        /// The maximum number of appeals until the court fails.
        #[pallet::constant]
        type MaxAppeals: Get<u32>;

        #[pallet::constant]
        type MaxJurors: Get<u32>;

        #[pallet::constant]
        type MinStake: Get<BalanceOf<Self>>;

        /// Identifier of this pallet
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Randomness source
        type Random: Randomness<Self::Hash, Self::BlockNumber>;

        /// Weight used to calculate the necessary staking amount to become a juror
        #[pallet::constant]
        type StakeWeight: Get<BalanceOf<Self>>;

        /// Slashed funds are send to the treasury
        #[pallet::constant]
        type TreasuryPalletId: Get<PalletId>;

        /// Weights generated by benchmarks
        type WeightInfo: WeightInfoZeitgeist;
    }

    // Number of jurors for an initial market dispute
    const INITIAL_JURORS_NUM: usize = 3;
    const MAX_RANDOM_JURORS: usize = 13;
    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);
    // Weight used to increase the number of jurors for subsequent disputes
    // of the same market
    const SUBSEQUENT_JURORS_FACTOR: usize = 2;
    // Divides the reserved juror balance to calculate the slash amount. `5` here
    // means that the output value will be 20% of the dividend.
    const TARDY_PUNISHMENT_DIVISOR: u8 = 5;

    pub(crate) type BalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type CurrencyOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Currency;
    pub(crate) type NegativeImbalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub(crate) type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;
    pub(crate) type MarketOf<T> = Market<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        <T as frame_system::Config>::BlockNumber,
        MomentOf<T>,
        Asset<MarketIdOf<T>>,
    >;
    pub(crate) type CourtOf<T> = CourtInfo<BalanceOf<T>, <T as frame_system::Config>::BlockNumber>;
    pub(crate) type JurorInfoOf<T> = JurorInfo<BalanceOf<T>>;
    pub(crate) type JurorListOf<T> = BoundedVec<
        (BalanceOf<T>, <T as frame_system::Config>::AccountId),
        <T as Config>::MaxJurors,
    >;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    /// Accounts that stake funds to decide outcomes.
    #[pallet::storage]
    pub type JurorList<T: Config> = StorageValue<_, JurorListOf<T>, ValueQuery>;

    #[pallet::storage]
    pub type Jurors<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, JurorInfoOf<T>, OptionQuery>;

    /// An extra layer of pseudo randomness.
    #[pallet::storage]
    pub type JurorsSelectionNonce<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    pub type RequestedVotes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        MarketIdOf<T>,
        Blake2_128Concat,
        T::AccountId,
        Vote<T::Hash>,
    >;

    #[pallet::storage]
    pub type Courts<T: Config> =
        StorageMap<_, Blake2_128Concat, MarketIdOf<T>, CourtOf<T>, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// A juror has been added to the court.
        JoinedJuror { juror: T::AccountId },
        /// A juror has been removed from the court.
        ExitedJuror { juror: T::AccountId },
        /// A juror has voted in a court.
        JurorVoted { market_id: MarketIdOf<T>, juror: T::AccountId, secret: T::Hash },
        /// A juror has revealed their vote.
        JurorRevealedVote {
            juror: T::AccountId,
            market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
            salt: T::Hash,
        },
        /// The jurors for an appeal have been drawn.
        AppealJurorsDrawn { market_id: MarketIdOf<T> },
        /// The crowdfund for an appeal has been checked.
        AppealCrowdfundChecked { market_id: MarketIdOf<T> },
        /// A market has been appealed.
        MarketAppealed { market_id: MarketIdOf<T>, appeal_number: u8 },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// An account id does not exist on the jurors storage.
        JurorDoesNotExists,
        /// On dispute or resolution, someone tried to pass a non-court market type
        MarketDoesNotHaveCourtMechanism,
        /// No-one voted on an outcome to resolve a market
        NoVotes,
        /// Forbids voting of unknown accounts
        OnlyJurorsCanVote,
        /// The market is not in a state where it can be disputed.
        MarketIsNotDisputed,
        /// Only jurors can reveal their votes.
        OnlyJurorsCanReveal,
        /// The vote was not found.
        NoVoteFound,
        /// The vote is not secret.
        VoteIsNotSecret,
        /// The outcome and salt reveal do not match the secret vote.
        InvalidReveal,
        /// The revealed vote outcome was not crowdfunded.
        InvalidCrowdfundItem,
        /// No court for this market id was found.
        CourtNotFound,
        /// This operation is only allowed in the voting period.
        NotInVotingPeriod,
        /// This operation is only allowed in the aggregation period.
        NotInAggregationPeriod,
        /// There is not enough crowdfund backing to appeal.
        NotEnoughCrowdfundBackingToAppeal,
        /// The maximum number of appeals has been reached.
        MaxAppealsReached,
        /// This operation is only allowed in the appeal period.
        NotInAppealPeriod,
        /// The court is already present for this market.
        CourtAlreadyExists,
        /// The revealed outcome is below the minimum threshold for the crowdfund.
        OutcomeCrowdfundsBelowThreshold,
        JurorsAlreadyDrawn,
        AppealAlreadyFunded,
        CheckCrowdfundFirst,
        AppealNotReady,
        OnlyDrawnJurorsCanVote,
        BelowMinStake,
        MaxJurorsReached,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(T::WeightInfo::join_court())]
        #[transactional]
        pub fn join_court(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(amount >= T::MinStake::get(), Error::<T>::BelowMinStake);

            let mut jurors = JurorList::<T>::get();

            let mut juror_info = JurorInfoOf::<T> { stake: amount };

            if let Some(prev_juror_info) = <Jurors<T>>::get(&who) {
                if let Ok(i) = jurors.binary_search_by_key(&prev_juror_info.stake, |tuple| tuple.0)
                {
                    jurors.remove(i);
                } else {
                    log::warn!("Join court: Juror stake not found in list");
                    debug_assert!(false);
                }

                let updated_stake = prev_juror_info.stake.saturating_add(amount);
                match jurors.binary_search_by_key(&updated_stake, |tuple| tuple.0) {
                    Ok(i) => jurors
                        .try_insert(i, (updated_stake, who.clone()))
                        .map_err(|_| Error::<T>::MaxJurorsReached)?,
                    Err(i) => jurors
                        .try_insert(i, (updated_stake, who.clone()))
                        .map_err(|_| Error::<T>::MaxJurorsReached)?,
                };

                juror_info.stake = updated_stake;
            } else {
                match jurors.binary_search_by_key(&amount, |tuple| tuple.0) {
                    Ok(i) => jurors
                        .try_insert(i, (amount, who.clone()))
                        .map_err(|_| Error::<T>::MaxJurorsReached)?,
                    Err(i) => jurors
                        .try_insert(i, (amount, who.clone()))
                        .map_err(|_| Error::<T>::MaxJurorsReached)?,
                };
            }

            CurrencyOf::<T>::reserve_named(&Self::reserve_id(), &who, amount)?;

            JurorList::<T>::put(jurors);

            <Jurors<T>>::insert(&who, juror_info);

            Self::deposit_event(Event::JoinedJuror { juror: who });
            Ok(())
        }

        #[pallet::weight(T::WeightInfo::exit_court())]
        #[transactional]
        pub fn exit_court(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::juror(&who)?;

            let mut jurors = JurorList::<T>::get();

            if let Some(prev_juror_info) = <Jurors<T>>::get(&who) {
                if let Ok(i) = jurors.binary_search_by_key(&prev_juror_info.stake, |tuple| tuple.0)
                {
                    jurors.remove(i);
                    <JurorList<T>>::put(jurors);
                } else {
                    log::warn!("Exit court: Juror stake not found in list");
                    debug_assert!(false);
                }
            }

            Self::remove_juror_from_all_courts_of_all_markets(&who);

            Self::deposit_event(Event::ExitedJuror { juror: who });
            Ok(())
        }

        #[pallet::weight(T::WeightInfo::vote())]
        #[transactional]
        pub fn vote(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            secret_vote: T::Hash,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(<Jurors<T>>::contains_key(&who), Error::<T>::OnlyJurorsCanVote);
            ensure!(
                <RequestedVotes<T>>::contains_key(&market_id, &who),
                Error::<T>::OnlyDrawnJurorsCanVote
            );

            let court = <Courts<T>>::get(&market_id).ok_or(Error::<T>::CourtNotFound)?;
            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(
                court.periods.crowdfund_end < now && now <= court.periods.vote_end,
                Error::<T>::NotInVotingPeriod
            );

            let vote = Vote::Secret { secret: secret_vote };
            RequestedVotes::<T>::insert(&market_id, &who, vote);

            Self::deposit_event(Event::JurorVoted { juror: who, market_id, secret: secret_vote });
            Ok(())
        }

        // TODO benchmark
        #[pallet::weight(1_000_000_000_000)]
        #[transactional]
        pub fn reveal_vote(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
            salt: T::Hash,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            if Jurors::<T>::get(&who).is_none() {
                return Err(Error::<T>::OnlyJurorsCanReveal.into());
            }
            let vote = <RequestedVotes<T>>::get(market_id, &who).ok_or(Error::<T>::NoVoteFound)?;
            let court = <Courts<T>>::get(&market_id).ok_or(Error::<T>::CourtNotFound)?;
            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(
                court.periods.vote_end < now && now <= court.periods.aggregation_end,
                Error::<T>::NotInAggregationPeriod
            );

            let fund_amount = T::Crowdfund::get_item(court.crowdfund_info.index, &outcome)
                .ok_or(Error::<T>::InvalidCrowdfundItem)?;
            ensure!(
                fund_amount >= court.crowdfund_info.threshold,
                Error::<T>::OutcomeCrowdfundsBelowThreshold
            );

            let secret = match vote {
                Vote::Secret { secret } => {
                    // market id and current appeal number is part of salt generation
                    // salt should be signed by the juror (market_id ++ appeal number)
                    // salt can be reproduced only be the juror address
                    // with knowing market_id and appeal number
                    // so even if the salt is forgotten it can be reproduced only by the juror
                    ensure!(
                        secret == T::Hashing::hash_of(&(who.clone(), outcome.clone(), salt)),
                        Error::<T>::InvalidReveal
                    );
                    secret
                }
                _ => return Err(Error::<T>::VoteIsNotSecret.into()),
            };

            let raw_vote = Vote::Revealed { secret, outcome: outcome.clone(), salt };
            RequestedVotes::<T>::insert(&market_id, &who, raw_vote);

            Self::deposit_event(Event::JurorRevealedVote { juror: who, market_id, outcome, salt });
            Ok(())
        }

        // TODO benchmark
        #[pallet::weight(1_000_000_000_000)]
        #[transactional]
        pub fn check_appeal_crowdfund(
            origin: OriginFor<T>,
            market_id: MarketIdOf<T>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            let mut court = <Courts<T>>::get(&market_id).ok_or(Error::<T>::CourtNotFound)?;
            ensure!(!court.appeal_info.is_funded, Error::<T>::AppealAlreadyFunded);
            let now = <frame_system::Pallet<T>>::block_number();
            Self::check_appealable_market(&market_id, &court, now)?;

            // update crowdfund threshold
            let threshold =
                court.crowdfund_info.threshold.saturating_add(court.crowdfund_info.threshold);

            let mut count = 0u32;
            // TODO: use iter_from https://paritytech.github.io/substrate/master/frame_support/pallet_prelude/struct.StorageMap.html#method.iter_from
            // TODO: with iter_from we can iterate from the last checked item (weight restrictions)
            for (_, crowdfund_amount) in T::Crowdfund::iter_items(court.crowdfund_info.index) {
                if crowdfund_amount >= threshold {
                    count = count.saturating_add(1);
                    if count >= 2 {
                        break;
                    }
                }
            }
            ensure!(count >= 2, Error::<T>::NotEnoughCrowdfundBackingToAppeal);

            court.crowdfund_info.threshold = threshold;
            court.appeal_info.is_funded = true;
            <Courts<T>>::insert(&market_id, court);

            Self::deposit_event(Event::AppealCrowdfundChecked { market_id });

            Ok(())
        }

        // TODO benchmark
        #[pallet::weight(1_000_000_000_000)]
        #[transactional]
        pub fn draw_appeal_jurors(
            origin: OriginFor<T>,
            market_id: MarketIdOf<T>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            let mut court = <Courts<T>>::get(&market_id).ok_or(Error::<T>::CourtNotFound)?;
            ensure!(!court.appeal_info.is_drawn, Error::<T>::JurorsAlreadyDrawn);
            ensure!(court.appeal_info.is_funded, Error::<T>::CheckCrowdfundFirst);
            let now = <frame_system::Pallet<T>>::block_number();
            Self::check_appealable_market(&market_id, &court, now)?;

            let appeal_number = court.appeal_info.current as usize;
            Self::select_jurors(&market_id, appeal_number);

            court.appeal_info.is_drawn = true;
            <Courts<T>>::insert(&market_id, court);

            Self::deposit_event(Event::AppealJurorsDrawn { market_id });

            Ok(())
        }

        // TODO benchmark
        #[pallet::weight(1_000_000_000_000)]
        #[transactional]
        pub fn appeal(origin: OriginFor<T>, market_id: MarketIdOf<T>) -> DispatchResult {
            ensure_signed(origin)?;

            let mut court = <Courts<T>>::get(&market_id).ok_or(Error::<T>::CourtNotFound)?;
            ensure!(court.appeal_info.is_appeal_ready(), Error::<T>::AppealNotReady);
            let now = <frame_system::Pallet<T>>::block_number();
            Self::check_appealable_market(&market_id, &court, now)?;

            let last_resolve_at = court.periods.appeal_end;
            let _ids_len_0 = T::DisputeResolution::remove_auto_resolve(&market_id, last_resolve_at);

            let periods = Periods {
                crowdfund_end: T::CourtCrowdfundPeriod::get(),
                vote_end: T::CourtVotePeriod::get(),
                aggregation_end: T::CourtAggregationPeriod::get(),
                appeal_end: T::CourtAppealPeriod::get(),
            };
            // sets periods one after the other from now
            court.update_periods(periods, now);
            let appeal_number = court.appeal_info.current;
            court.appeal_info.current = court.appeal_info.current.saturating_add(1);

            let _ids_len_1 =
                T::DisputeResolution::add_auto_resolve(&market_id, court.periods.appeal_end)?;

            <Courts<T>>::insert(market_id, court);

            Self::deposit_event(Event::MarketAppealed { market_id, appeal_number });

            Ok(())
        }
    }

    impl<T> Pallet<T>
    where
        T: Config,
    {
        pub(crate) fn select_jurors(market_id: &MarketIdOf<T>, appeal_number: usize) {
            let jurors: Vec<_> = Jurors::<T>::iter().collect();
            let necessary_jurors_num = Self::necessary_jurors_num(appeal_number);
            let mut rng = Self::rng();
            let random_jurors = Self::random_jurors(&jurors, necessary_jurors_num, &mut rng);
            for (ai, _) in random_jurors {
                RequestedVotes::<T>::insert(market_id, ai, Vote::Drawn);
            }
        }

        pub(crate) fn check_appealable_market(
            market_id: &MarketIdOf<T>,
            court: &CourtOf<T>,
            now: T::BlockNumber,
        ) -> Result<(), DispatchError> {
            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.status == MarketStatus::Disputed, Error::<T>::MarketIsNotDisputed);
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );

            ensure!(
                court.periods.aggregation_end < now && now <= court.periods.appeal_end,
                Error::<T>::NotInAppealPeriod
            );

            ensure!(
                court.appeal_info.current < court.appeal_info.max,
                Error::<T>::MaxAppealsReached
            );

            Ok(())
        }

        // Returns an unique random subset of `jurors` with length `len`.
        //
        // If `len` is greater than the length of `jurors`, then `len` will be capped.
        pub(crate) fn random_jurors<'a, 'b, R>(
            jurors: &'a [(T::AccountId, JurorInfoOf<T>)],
            len: usize,
            rng: &mut R,
        ) -> ArrayVec<&'b (T::AccountId, JurorInfoOf<T>), MAX_RANDOM_JURORS>
        where
            'a: 'b,
            R: RngCore,
        {
            let actual_len = jurors.len().min(len);
            jurors.choose_multiple(rng, actual_len).collect()
        }

        /// The reserve ID of the court pallet.
        #[inline]
        pub fn reserve_id() -> [u8; 8] {
            T::PalletId::get().0
        }

        // Returns a pseudo random number generator implementation based on the seed
        // provided by the `Config::Random` type and the `JurorsSelectionNonce` storage.
        pub(crate) fn rng() -> impl RngCore {
            let nonce = <JurorsSelectionNonce<T>>::mutate(|n| {
                let rslt = *n;
                *n = n.wrapping_add(1);
                rslt
            });
            let mut seed = [0; 32];
            let (random_hash, _) = T::Random::random(&nonce.to_le_bytes());
            for (byte, el) in random_hash.as_ref().iter().copied().zip(seed.iter_mut()) {
                *el = byte
            }
            StdRng::from_seed(seed)
        }

        #[inline]
        pub(crate) fn treasury_account_id() -> T::AccountId {
            T::TreasuryPalletId::get().into_account_truncating()
        }

        // Retrieves a juror from the storage
        fn juror(account_id: &T::AccountId) -> Result<JurorInfoOf<T>, DispatchError> {
            Jurors::<T>::get(account_id).ok_or_else(|| Error::<T>::JurorDoesNotExists.into())
        }

        // Calculates the necessary number of jurors depending on the number of market disputes.
        //
        // Result is capped to `usize::MAX` or in other words, capped to a very, very, very
        // high number of jurors.
        fn necessary_jurors_num(appeals_len: usize) -> usize {
            // TODO: increase requested juror number higher; current max example: 3 + 2 * MaxAppeals
            INITIAL_JURORS_NUM.saturating_add(SUBSEQUENT_JURORS_FACTOR.saturating_mul(appeals_len))
        }

        // Every juror that not voted on the first or second most voted outcome are slashed.
        fn slash_losers_to_award_winners(
            valid_winners_and_losers: &[(&T::AccountId, &OutcomeReport)],
            winner_outcome: &OutcomeReport,
        ) -> DispatchResult {
            let mut total_incentives = BalanceOf::<T>::from(0u8);
            let mut total_winners = BalanceOf::<T>::from(0u8);

            let mut winners = Vec::with_capacity(valid_winners_and_losers.len());
            for (juror, outcome) in valid_winners_and_losers {
                if outcome == &winner_outcome {
                    winners.push(juror);
                    total_winners = total_winners.saturating_add(BalanceOf::<T>::from(1u8));
                } else {
                    let all_reserved =
                        CurrencyOf::<T>::reserved_balance_named(&Self::reserve_id(), juror);
                    // Unsigned division will never overflow
                    let slash = all_reserved
                        .checked_div(&BalanceOf::<T>::from(2u8))
                        .ok_or(DispatchError::Other("Zero division"))?;
                    CurrencyOf::<T>::slash_reserved_named(&Self::reserve_id(), juror, slash);
                    total_incentives = total_incentives.saturating_add(slash);
                }
            }

            let individual_winner_incentive =
                if let Some(e) = total_incentives.checked_div(&total_winners) {
                    e
                } else {
                    // No winners
                    return Ok(());
                };

            for juror in winners {
                CurrencyOf::<T>::deposit_into_existing(juror, individual_winner_incentive)?;
            }

            Ok(())
        }

        fn aggregate(
            votes: &[(T::AccountId, Vote<T::Hash>)],
        ) -> Result<(OutcomeReport, Vec<(&T::AccountId, &OutcomeReport)>), DispatchError> {
            let mut scores = BTreeMap::<OutcomeReport, u32>::new();

            let treasury_account_id = Self::treasury_account_id();

            let slash_and_remove_juror = |ai: &T::AccountId| {
                let all_reserved = CurrencyOf::<T>::reserved_balance_named(&Self::reserve_id(), ai);
                // Unsigned division will never overflow
                let slash = all_reserved
                    .checked_div(&BalanceOf::<T>::from(TARDY_PUNISHMENT_DIVISOR))
                    .ok_or(DispatchError::Other("Zero division"))?;
                let _ = CurrencyOf::<T>::repatriate_reserved_named(
                    &Self::reserve_id(),
                    ai,
                    &treasury_account_id,
                    slash,
                    BalanceStatus::Free,
                )?;
                Self::remove_juror_from_all_courts_of_all_markets(ai);
                Ok::<_, DispatchError>(())
            };

            let mut valid_winners_and_losers = Vec::with_capacity(votes.len());

            for (juror, vote) in votes {
                match vote {
                    Vote::Drawn => {
                        slash_and_remove_juror(juror)?;
                    }
                    Vote::Secret { secret: _ } => {
                        slash_and_remove_juror(juror)?;
                    }
                    Vote::Revealed { secret: _, outcome, salt: _ } => {
                        if let Some(el) = scores.get_mut(&outcome) {
                            *el = el.saturating_add(1);
                        } else {
                            scores.insert(outcome.clone(), 1);
                        }
                        valid_winners_and_losers.push((juror, outcome));
                    }
                }
            }

            let mut iter = scores.iter();

            let mut best_score = if let Some(first) = iter.next() {
                first
            } else {
                // TODO this should never happen, we should have another vote round for it
                // TODO: the appeal round should be repeated
                // TODO: right after each aggregation period `on_initialize` should check if we have a clear winner (one outcome with plurality of votes) and at least one revealed vote
                // TODO: if there is no clear winner, the appeal should be repeated (same appeal number)
                return Err(Error::<T>::NoVotes.into());
            };

            for el in iter {
                if el.1 > best_score.1 {
                    best_score = el;
                }
            }

            Ok((best_score.0.clone(), valid_winners_and_losers))
        }

        // Obliterates all stored references of a juror un-reserving balances.
        fn remove_juror_from_all_courts_of_all_markets(ai: &T::AccountId) {
            CurrencyOf::<T>::unreserve_all_named(&Self::reserve_id(), ai);
            Jurors::<T>::remove(ai);
            let mut market_ids = BTreeSet::new();
            market_ids.extend(RequestedVotes::<T>::iter().map(|el| el.0));
            for market_id in &market_ids {
                // TODO: should we allow removing all vote requests?
                // TODO: because the juror could avoid getting slashed by just exit_court
                RequestedVotes::<T>::remove(market_id, ai);
            }
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

        fn on_dispute(market_id: &Self::MarketId, market: &MarketOf<T>) -> DispatchResult {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );

            ensure!(!<Courts<T>>::contains_key(market_id), Error::<T>::CourtAlreadyExists);

            let now = <frame_system::Pallet<T>>::block_number();
            let index = T::Crowdfund::open_crowdfund()?;

            let crowdfund_info =
                CrowdfundInfo { index, threshold: T::CrowdfundMinThreshold::get() };

            let periods = Periods {
                crowdfund_end: T::CourtCrowdfundPeriod::get(),
                vote_end: T::CourtVotePeriod::get(),
                aggregation_end: T::CourtAggregationPeriod::get(),
                appeal_end: T::CourtAppealPeriod::get(),
            };

            // sets periods one after the other from now
            let court = CourtInfo::new(crowdfund_info, now, periods, T::MaxAppeals::get() as u8);

            Self::select_jurors(market_id, 0usize);

            let _ids_len =
                T::DisputeResolution::add_auto_resolve(&market_id, court.periods.appeal_end)?;

            <Courts<T>>::insert(market_id, court);

            Ok(())
        }

        fn get_resolution_outcome(
            market_id: &Self::MarketId,
            market: &MarketOf<T>,
        ) -> Result<Option<OutcomeReport>, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );
            let votes: Vec<_> = RequestedVotes::<T>::iter_prefix(market_id).collect();
            let (winner_outcome, valid_winners_and_losers) = Self::aggregate(votes.as_slice())?;
            Self::slash_losers_to_award_winners(&valid_winners_and_losers, &winner_outcome)?;

            let court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;

            // TODO: use an own API call instead, which allows to prepare the refund for all inside the crowdfund pallet (call extrinsic multiple times)
            // TODO: specify fees somehow for specific outcomes in this api call
            // TODO: the reason for this is that there are weight limitations here (storage iter)
            for (outcome, _crowdfund_amount) in T::Crowdfund::iter_items(court.crowdfund_info.index)
            {
                T::Crowdfund::prepare_refund(
                    court.crowdfund_info.index,
                    &outcome,
                    Percent::zero(),
                )?;
            }
            T::Crowdfund::close_crowdfund(court.crowdfund_info.index)?;

            // this only removes a maximum of jurors according to MaxAppeals (weight limited)
            let _ = RequestedVotes::<T>::clear_prefix(market_id, u32::max_value(), None);

            Ok(Some(winner_outcome))
        }

        fn maybe_pay(
            _: &Self::MarketId,
            market: &MarketOf<T>,
            _: &OutcomeReport,
            overall_imbalance: NegativeImbalanceOf<T>,
        ) -> Result<NegativeImbalanceOf<T>, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );
            // TODO all funds to treasury?
            Ok(overall_imbalance)
        }

        fn get_auto_resolve(
            market_id: &Self::MarketId,
            market: &MarketOf<T>,
        ) -> Result<Option<Self::BlockNumber>, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );

            let court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;
            Ok(Some(court.periods.appeal_end))
        }

        fn has_failed(
            market_id: &Self::MarketId,
            market: &MarketOf<T>,
        ) -> Result<bool, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );

            let court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;
            Ok(court.appeal_info.current >= court.appeal_info.max)
        }

        fn on_global_dispute(_: &Self::MarketId, market: &MarketOf<T>) -> DispatchResult {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );
            Ok(())
        }

        fn clear(market_id: &Self::MarketId, market: &MarketOf<T>) -> DispatchResult {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );
            // only removes a maximum of jurors according to MaxAppeals (weight limited)
            let _ = RequestedVotes::<T>::clear_prefix(market_id, u32::max_value(), None);
            <Courts<T>>::remove(market_id);
            Ok(())
        }
    }

    impl<T> CourtPalletApi for Pallet<T> where T: Config {}
}
