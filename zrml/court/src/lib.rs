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
pub use types::{CourtInfo, CrowdfundInfo, Juror, JurorStatus, Periods, Vote};

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
        Juror, JurorStatus, Periods, Vote,
    };
    use alloc::{
        collections::{BTreeMap, BTreeSet},
        vec::Vec,
    };
    use arrayvec::ArrayVec;
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::DispatchResult,
        ensure,
        pallet_prelude::{
            CountedStorageMap, OptionQuery, StorageDoubleMap, StorageMap, StorageValue, ValueQuery,
        },
        traits::{
            BalanceStatus, Currency, Get, Hooks, IsType, NamedReservableCurrency, Randomness,
            StorageVersion,
        },
        transactional, Blake2_128Concat, PalletId,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use rand::{rngs::StdRng, seq::SliceRandom, RngCore, SeedableRng};
    use sp_runtime::{
        traits::{AccountIdConversion, CheckedDiv, Hash, Saturating},
        ArithmeticError, DispatchError, Percent, SaturatedConversion,
    };
    use zeitgeist_primitives::{
        traits::{DisputeApi, DisputeResolutionApi},
        types::{Asset, Market, MarketDisputeMechanism, MarketStatus, OutcomeReport},
    };
    use zrml_market_commons::MarketCommonsPalletApi;

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

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(T::WeightInfo::join_court())]
        #[transactional]
        pub fn join_court(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            if Jurors::<T>::get(&who).is_some() {
                return Err(Error::<T>::JurorAlreadyExists.into());
            }
            let jurors_num = Jurors::<T>::count() as usize;
            let jurors_num_plus_one = jurors_num.checked_add(1).ok_or(ArithmeticError::Overflow)?;
            let stake = Self::current_required_stake(jurors_num_plus_one);
            CurrencyOf::<T>::reserve_named(&Self::reserve_id(), &who, stake)?;
            let juror = Juror { status: JurorStatus::Ok };
            Jurors::<T>::insert(&who, juror.clone());
            Self::deposit_event(Event::JoinedJuror(who, juror));
            Ok(())
        }

        #[pallet::weight(T::WeightInfo::exit_court())]
        #[transactional]
        pub fn exit_court(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let juror = Self::juror(&who)?;
            Self::remove_juror_from_all_courts_of_all_markets(&who);
            Self::deposit_event(Event::ExitedJuror(who, juror));
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
            if Jurors::<T>::get(&who).is_none() {
                return Err(Error::<T>::OnlyJurorsCanVote.into());
            }

            let court = <Courts<T>>::get(&market_id).ok_or(Error::<T>::CourtNotFound)?;
            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(
                court.periods.crowdfund_end < now && now <= court.periods.vote_end,
                Error::<T>::NotInVotingPeriod
            );

            let vote = Vote::Secret { secret: secret_vote };
            Votes::<T>::insert(market_id, who, vote);
            Ok(())
        }

        // TODO benchmark
        #[pallet::weight(T::WeightInfo::vote())]
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
            let vote = <Votes<T>>::get(market_id, &who).ok_or(Error::<T>::NoVoteFound)?;
            let court = <Courts<T>>::get(&market_id).ok_or(Error::<T>::CourtNotFound)?;
            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(
                court.periods.vote_end < now && now <= court.periods.aggregation_end,
                Error::<T>::NotInAggregationPeriod
            );

            // TODO maybe check here if fund amount does fulfill the required stake
            let fund_amount = T::Crowdfund::get_item(court.crowdfund_info.index, &outcome)
                .ok_or(Error::<T>::InvalidCrowdfundItem)?;
            ensure!(
                fund_amount >= court.crowdfund_info.threshold,
                Error::<T>::OutcomeCrowdfundsBelowThreshold
            );

            let secret = match vote {
                Vote::Secret { secret } => {
                    ensure!(
                        secret == T::Hashing::hash_of(&(who, market_id, outcome, salt)),
                        Error::<T>::InvalidReveal
                    );
                    secret
                }
                _ => return Err(Error::<T>::VoteIsNotSecret.into()),
            };

            let raw_vote = Vote::Revealed { secret, outcome, salt };
            Votes::<T>::insert(market_id, who, raw_vote);
            Ok(())
        }

        // TODO benchmark
        #[pallet::weight(1_000_000_000_000)]
        #[transactional]
        pub fn appeal(origin: OriginFor<T>, market_id: MarketIdOf<T>) -> DispatchResult {
            ensure_signed(origin)?;
            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.status == MarketStatus::Disputed, Error::<T>::MarketIsNotDisputed);
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );

            let mut court = <Courts<T>>::get(&market_id).ok_or(Error::<T>::CourtNotFound)?;
            let now = <frame_system::Pallet<T>>::block_number();

            ensure!(
                court.periods.aggregation_end < now && now <= court.periods.appeal_end,
                Error::<T>::NotInAppealPeriod
            );

            ensure!(
                court.appeal_info.current < court.appeal_info.max,
                Error::<T>::MaxAppealsReached
            );

            let iter = T::Crowdfund::iter_items(court.crowdfund_info.index);
            let mut count = 0u32;
            let mut funded_outcomes = Vec::new();
            let threshold = court.crowdfund_info.threshold;
            for (outcome, crowdfund_amount) in iter {
                if crowdfund_amount >= threshold {
                    funded_outcomes.push(outcome);
                    count = count.saturating_add(1);
                }
            }
            ensure!(count >= 2, Error::<T>::NotEnoughCrowdfundBackingToAppeal);

            let last_resolve_at = court.periods.appeal_end;
            let _ids_len_0 = T::DisputeResolution::remove_auto_resolve(&market_id, last_resolve_at);

            let periods = Periods {
                crowdfund_end: T::CourtCrowdfundPeriod::get(),
                vote_end: T::CourtVotePeriod::get(),
                aggregation_end: T::CourtAggregationPeriod::get(),
                appeal_end: T::CourtAppealPeriod::get(),
            };
            // sets periods one after the other from now
            court.appeal(periods, now);

            let jurors: Vec<_> = Jurors::<T>::iter().collect();
            let current_appeals = court.appeal_info.current as usize;
            let necessary_jurors_num =
                Self::necessary_jurors_num(current_appeals);
            let mut rng = Self::rng();
            let random_jurors = Self::random_jurors(&jurors, necessary_jurors_num, &mut rng);
            for (ai, _) in random_jurors {
                RequestedJurors::<T>::insert(market_id, ai, ());
            }

            let _ids_len_1 =
                T::DisputeResolution::add_auto_resolve(&market_id, court.periods.appeal_end)?;

            <Courts<T>>::insert(market_id, court);

            Self::deposit_event(Event::MarketAppealed { market_id });

            Ok(())
        }
    }

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

    #[pallet::error]
    pub enum Error<T> {
        /// It is not possible to insert a Juror that is already stored
        JurorAlreadyExists,
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
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        ExitedJuror(T::AccountId, Juror),
        JoinedJuror(T::AccountId, Juror),
        /// A market has been appealed.
        MarketAppealed {
            market_id: MarketIdOf<T>,
        },
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
        // Returns an unique random subset of `jurors` with length `len`.
        //
        // If `len` is greater than the length of `jurors`, then `len` will be capped.
        pub(crate) fn random_jurors<'a, 'b, R>(
            jurors: &'a [(T::AccountId, Juror)],
            len: usize,
            rng: &mut R,
        ) -> ArrayVec<&'b (T::AccountId, Juror), MAX_RANDOM_JURORS>
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

        // Used to avoid code duplications.
        pub(crate) fn set_stored_juror_as_tardy(account_id: &T::AccountId) -> DispatchResult {
            Self::mutate_juror(account_id, |juror| {
                juror.status = JurorStatus::Tardy;
                Ok(())
            })
        }

        #[inline]
        pub(crate) fn treasury_account_id() -> T::AccountId {
            T::TreasuryPalletId::get().into_account_truncating()
        }

        // No-one can stake more than BalanceOf::<T>::max(), therefore, this function saturates
        // arithmetic operations.
        fn current_required_stake(jurors_num: usize) -> BalanceOf<T> {
            let jurors_len: BalanceOf<T> = jurors_num.saturated_into();
            T::StakeWeight::get().saturating_mul(jurors_len)
        }

        // Retrieves a juror from the storage
        fn juror(account_id: &T::AccountId) -> Result<Juror, DispatchError> {
            Jurors::<T>::get(account_id).ok_or_else(|| Error::<T>::JurorDoesNotExists.into())
        }

        // # Manages tardy jurors and returns valid winners and valid losers.
        //
        // ## Management
        //
        // * Jurors that didn't vote within `CourtCaseDuration` or didn't vote at all are
        // placed as tardy.
        //
        // * Slashes 20% of staked funds and removes tardy jurors that didn't vote or voted
        // after the maximum allowed block.
        //
        // ## Returned list of accounts
        //
        // All new and old tardy jurors, excluding the ones that voted within `CourtCaseDuration`,
        // are removed from the list of accounts that will be slashed to reward winners. Already
        // tardy jurors that voted again on the second most voted outcome are also removed from the
        // same list.
        //
        // In other words, does not yield slashed accounts, winners of the losing side,
        // accounts that didn't vote or accounts that voted after the maximum allowed block
        fn manage_tardy_jurors<'a, 'b, F>(
            requested_jurors: &'a [(
                T::AccountId,
                Juror,
                T::BlockNumber,
                Option<&(T::BlockNumber, OutcomeReport)>,
            )],
            mut cb: F,
        ) -> Result<Vec<(&'b T::AccountId, &'b OutcomeReport)>, DispatchError>
        where
            F: FnMut(&OutcomeReport) -> bool,
            'a: 'b,
        {
            let mut valid_winners_and_losers = Vec::with_capacity(requested_jurors.len());
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

            for (ai, juror, max_block, vote_opt) in requested_jurors {
                if let Some((block, outcome)) = vote_opt {
                    let vote_is_expired = block > max_block;
                    if vote_is_expired {
                        // Tardy juror voted after maximum allowed block. Slash
                        if let JurorStatus::Tardy = juror.status {
                            slash_and_remove_juror(ai)?;
                        }
                        // Ordinary juror voted after maximum allowed block. Set as tardy
                        else {
                            Self::set_stored_juror_as_tardy(ai)?;
                        }
                    } else {
                        let has_voted_on_the_second_most_outcome = cb(outcome);
                        if has_voted_on_the_second_most_outcome {
                            // Don't set already tardy juror as tardy again
                            if JurorStatus::Tardy != juror.status {
                                Self::set_stored_juror_as_tardy(ai)?;
                            }
                        } else {
                            valid_winners_and_losers.push((ai, outcome));
                        }
                    }
                // Tardy juror didn't vote. Slash
                } else if let JurorStatus::Tardy = juror.status {
                    slash_and_remove_juror(ai)?;
                }
                // Ordinary juror didn't vote. Set as tardy
                else {
                    Self::set_stored_juror_as_tardy(ai)?;
                }
            }

            Ok(valid_winners_and_losers)
        }

        // Modifies a stored juror.
        fn mutate_juror<F>(account_id: &T::AccountId, mut cb: F) -> DispatchResult
        where
            F: FnMut(&mut Juror) -> DispatchResult,
        {
            Jurors::<T>::try_mutate(account_id, |opt| {
                if let Some(el) = opt {
                    cb(el)?;
                } else {
                    return Err(Error::<T>::JurorDoesNotExists.into());
                }
                Ok(())
            })
        }

        // Calculates the necessary number of jurors depending on the number of market disputes.
        //
        // Result is capped to `usize::MAX` or in other words, capped to a very, very, very
        // high number of jurors.
        fn necessary_jurors_num(appeals_len: usize) -> usize {
            INITIAL_JURORS_NUM.saturating_add(SUBSEQUENT_JURORS_FACTOR.saturating_mul(appeals_len))
        }

        // Every juror that not voted on the first or second most voted outcome are slashed.
        fn slash_losers_to_award_winners(
            valid_winners_and_losers: &[(&T::AccountId, &OutcomeReport)],
            winner_outcome: &OutcomeReport,
        ) -> DispatchResult {
            let mut total_incentives = BalanceOf::<T>::from(0u8);
            let mut total_winners = BalanceOf::<T>::from(0u8);

            for (jai, outcome) in valid_winners_and_losers {
                if outcome == &winner_outcome {
                    total_winners = total_winners.saturating_add(BalanceOf::<T>::from(1u8));
                } else {
                    let all_reserved =
                        CurrencyOf::<T>::reserved_balance_named(&Self::reserve_id(), jai);
                    // Unsigned division will never overflow
                    let slash = all_reserved
                        .checked_div(&BalanceOf::<T>::from(2u8))
                        .ok_or(DispatchError::Other("Zero division"))?;
                    CurrencyOf::<T>::slash_reserved_named(&Self::reserve_id(), jai, slash);
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

            for (jai, outcome) in valid_winners_and_losers {
                if outcome == &winner_outcome {
                    CurrencyOf::<T>::deposit_into_existing(jai, individual_winner_incentive)?;
                }
            }

            Ok(())
        }

        // For market resolution based on the votes of a market
        fn two_best_outcomes(
            votes: &[(T::AccountId, (T::BlockNumber, OutcomeReport))],
        ) -> Result<(OutcomeReport, Option<OutcomeReport>), DispatchError> {
            let mut scores = BTreeMap::<OutcomeReport, u32>::new();

            for (_, (_, outcome_report)) in votes {
                if let Some(el) = scores.get_mut(outcome_report) {
                    *el = el.saturating_add(1);
                } else {
                    scores.insert(outcome_report.clone(), 1);
                }
            }

            let mut iter = scores.iter();

            let mut best_score = if let Some(first) = iter.next() {
                first
            } else {
                return Err(Error::<T>::NoVotes.into());
            };

            let mut second_best_score = if let Some(second) = iter.next() {
                if second.1 > best_score.1 {
                    best_score = second;
                    best_score
                } else {
                    second
                }
            } else {
                return Ok((best_score.0.clone(), None));
            };

            for el in iter {
                if el.1 > best_score.1 {
                    best_score = el;
                    second_best_score = best_score;
                } else if el.1 > second_best_score.1 {
                    second_best_score = el;
                }
            }

            Ok((best_score.0.clone(), Some(second_best_score.0.clone())))
        }

        // Obliterates all stored references of a juror un-reserving balances.
        fn remove_juror_from_all_courts_of_all_markets(ai: &T::AccountId) {
            CurrencyOf::<T>::unreserve_all_named(&Self::reserve_id(), ai);
            Jurors::<T>::remove(ai);
            let mut market_ids = BTreeSet::new();
            market_ids.extend(RequestedJurors::<T>::iter().map(|el| el.0));
            for market_id in &market_ids {
                RequestedJurors::<T>::remove(market_id, ai);
            }
            market_ids.clear();
            market_ids.extend(Votes::<T>::iter().map(|el| el.0));
            for market_id in &market_ids {
                Votes::<T>::remove(market_id, ai);
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

            let _ids_len =
                T::DisputeResolution::add_auto_resolve(&market_id, court.periods.appeal_end)?;

            <Courts<T>>::insert(market_id, court);

            Ok(())
        }

        // Set jurors that sided on the second most voted outcome as tardy. Jurors are only
        // rewarded if sided on the most voted outcome but jurors that voted second most
        // voted outcome (winner of the losing majority) are placed as tardy instead of
        // being slashed.
        fn get_resolution_outcome(
            market_id: &Self::MarketId,
            market: &MarketOf<T>,
        ) -> Result<Option<OutcomeReport>, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );
            let votes: Vec<_> = Votes::<T>::iter_prefix(market_id).collect();
            let requested_jurors: Vec<_> = RequestedJurors::<T>::iter_prefix(market_id)
                .map(|(juror_id, ())| {
                    let juror = Self::juror(&juror_id)?;
                    let vote_opt = votes.iter().find(|el| el.0 == juror_id).map(|el| &el.1);
                    Ok((juror_id, juror, vote_opt))
                })
                .collect::<Result<_, DispatchError>>()?;
            let (first, second_opt) = Self::two_best_outcomes(&votes)?;
            let valid_winners_and_losers = if let Some(second) = second_opt {
                Self::manage_tardy_jurors(&requested_jurors, |outcome| outcome == &second)?
            } else {
                Self::manage_tardy_jurors(&requested_jurors, |_| false)?
            };
            Self::slash_losers_to_award_winners(&valid_winners_and_losers, &first)?;
            let _ = Votes::<T>::clear_prefix(market_id, u32::max_value(), None);
            let _ = RequestedJurors::<T>::clear_prefix(market_id, u32::max_value(), None);

            let court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;

            for (outcome, _crowdfund_amount) in T::Crowdfund::iter_items(court.crowdfund_info.index)
            {
                T::Crowdfund::prepare_refund(
                    court.crowdfund_info.index,
                    &outcome,
                    Percent::zero(),
                )?;
            }
            T::Crowdfund::close_crowdfund(court.crowdfund_info.index)?;

            Ok(Some(first))
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

            Ok(court.appeals as u32 >= T::MaxAppeals::get())
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
            let _ = Votes::<T>::clear_prefix(market_id, u32::max_value(), None);
            let _ = RequestedJurors::<T>::clear_prefix(market_id, u32::max_value(), None);
            Ok(())
        }
    }

    impl<T> CourtPalletApi for Pallet<T> where T: Config {}

    /// Accounts that stake funds to decide outcomes.
    #[pallet::storage]
    pub type Jurors<T: Config> = CountedStorageMap<_, Blake2_128Concat, T::AccountId, Juror>;

    /// An extra layer of pseudo randomness.
    #[pallet::storage]
    pub type JurorsSelectionNonce<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Selected jurors that should vote a market outcome until a certain block number
    #[pallet::storage]
    pub type RequestedJurors<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, MarketIdOf<T>, Blake2_128Concat, T::AccountId, ()>;

    /// Votes of market outcomes for disputes
    ///
    /// Stores the vote block number and the submitted outcome.
    #[pallet::storage]
    pub type Votes<T: Config> = StorageDoubleMap<
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
}
