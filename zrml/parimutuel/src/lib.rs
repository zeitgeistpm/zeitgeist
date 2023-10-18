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

mod benchmarks;
mod mock;
mod tests;
pub mod weights;

pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::weights::WeightInfoZeitgeist;
    use alloc::{vec, vec::Vec};
    use core::marker::PhantomData;
    use frame_support::{
        ensure,
        pallet_prelude::{DispatchError, OptionQuery, StorageMap},
        traits::{Get, IsType, StorageVersion},
        PalletId, Twox64Concat,
    };
    use frame_system::{
        ensure_signed,
        pallet_prelude::{BlockNumberFor, OriginFor},
    };
    use orml_traits::MultiCurrency;
    use sp_runtime::{
        traits::{AccountIdConversion, CheckedSub, Zero},
        DispatchResult, SaturatedConversion, Saturating,
    };
    use zeitgeist_primitives::{
        constants::BASE,
        math::fixed::*,
        traits::DistributeFees,
        types::{Asset, Market, MarketStatus, MarketType, OutcomeReport, ScoringRule},
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The way how fees are taken from the market base asset collateral.
        type ExternalFees: DistributeFees<
                Asset = Asset<MarketIdOf<Self>>,
                AccountId = AccountIdOf<Self>,
                Balance = BalanceOf<Self>,
                MarketId = MarketIdOf<Self>,
            >;

        /// Event
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The market api.
        type MarketCommons: MarketCommonsPalletApi<
                AccountId = Self::AccountId,
                BlockNumber = Self::BlockNumber,
                Balance = BalanceOf<Self>,
            >;

        /// The api to handle different asset classes.
        type AssetManager: MultiCurrency<Self::AccountId, CurrencyId = AssetOf<Self>>;

        /// The minimum amount each bet must be. Must be larger than the existential deposit of parimutuel shares.
        #[pallet::constant]
        type MinBetSize: Get<BalanceOf<Self>>;

        /// Identifier of this pallet
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Weights generated by benchmarks
        type WeightInfo: WeightInfoZeitgeist;
    }

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

    pub(crate) type AssetOf<T> = Asset<MarketIdOf<T>>;
    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type BalanceOf<T> =
        <<T as Config>::AssetManager as MultiCurrency<AccountIdOf<T>>>::Balance;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub(crate) type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;
    pub(crate) type MarketOf<T> =
        Market<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>, MomentOf<T>, Asset<MarketIdOf<T>>>;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    /// The total rewards for each market at the market close.
    #[pallet::storage]
    pub type TotalRewards<T: Config> =
        StorageMap<_, Twox64Concat, MarketIdOf<T>, BalanceOf<T>, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// An outcome was bought.
        OutcomeBought {
            market_id: MarketIdOf<T>,
            buyer: AccountIdOf<T>,
            asset: AssetOf<T>,
            amount_minus_fees: BalanceOf<T>,
            fees: BalanceOf<T>,
        },
        /// Rewards of the pot were claimed.
        RewardsClaimed {
            market_id: MarketIdOf<T>,
            asset: AssetOf<T>,
            balance: BalanceOf<T>,
            actual_payoff: BalanceOf<T>,
            sender: AccountIdOf<T>,
        },
        /// A market base asset collateral was refunded.
        BalanceRefunded {
            market_id: MarketIdOf<T>,
            asset: AssetOf<T>,
            refunded_balance: BalanceOf<T>,
            sender: AccountIdOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// There was no buyer of the winning outcome.
        /// Use the `refund` extrinsic to get the initial bet back.
        NoWinner,
        /// The market is not active.
        MarketIsNotActive,
        /// The specified amount is below the minimum bet size.
        AmountTooSmall,
        /// The specified asset is not a parimutuel share.
        NotParimutuelOutcome,
        /// The specified asset was not found in the market assets.
        InvalidOutcomeAsset,
        /// The scoring rule is not parimutuel.
        InvalidScoringRule,
        /// The specified amount can not be transferred.
        InsufficientBalance,
        /// The market is not resolved yet.
        MarketIsNotResolvedYet,
        /// An unexpected error occured. This should never happen!
        /// There was an internal coding mistake.
        Unexpected,
        /// There is no resolved outcome present for the market.
        NoResolvedOutcome,
        /// The refund is not allowed.
        RefundNotAllowed,
        /// There is no balance to refund.
        RefundableBalanceIsZero,
        /// There is no reward, because there are no winning shares.
        NoWinningShares,
        /// There are not enough funds in the pot to reward the calculated amount.
        /// This should never happen, but if it does, we have an accounting problem.
        InsufficientFundsInPotAccount,
        /// The outcome issuance is greater than the market base asset collateral.
        /// This should never happen, but if it does, we have an accounting problem.
        OutcomeIssuanceGreaterCollateral,
        /// A scalar market is not allowed for parimutuels.
        ScalarMarketsNotAllowed,
        /// Only categorical markets are allowed for parimutuels.
        OnlyCategoricalMarketsAllowed,
        /// There is no reward to distribute.
        NoRewardToDistribute,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Buy parimutuel shares for markets base asset collateral.
        ///
        /// # Arguments
        ///
        /// - `asset`: The outcome asset to buy the shares of.
        /// - `amount`: The amount of collateral (base asset) to spend
        /// and of parimutuel shares to receive.
        /// Keep in mind that market creator fees are taken from this amount.
        ///
        /// Complexity: `O(log(n))`, where `n` is the number of assets in the market.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::buy())]
        #[frame_support::transactional]
        pub fn buy(
            origin: OriginFor<T>,
            asset: Asset<MarketIdOf<T>>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(amount >= T::MinBetSize::get(), Error::<T>::AmountTooSmall);

            let market_id = match asset {
                Asset::ParimutuelShare(market_id, _) => market_id,
                _ => return Err(Error::<T>::NotParimutuelOutcome.into()),
            };
            let market = T::MarketCommons::market(&market_id)?;
            let base_asset = market.base_asset;
            ensure!(
                T::AssetManager::ensure_can_withdraw(base_asset, &who, amount).is_ok(),
                Error::<T>::InsufficientBalance
            );
            ensure!(market.status == MarketStatus::Active, Error::<T>::MarketIsNotActive);
            ensure!(market.scoring_rule == ScoringRule::Parimutuel, Error::<T>::InvalidScoringRule);
            ensure!(
                matches!(market.market_type, MarketType::Categorical(_)),
                Error::<T>::OnlyCategoricalMarketsAllowed
            );
            let market_assets = Self::outcome_assets(market_id, &market)?;
            ensure!(market_assets.binary_search(&asset).is_ok(), Error::<T>::InvalidOutcomeAsset);

            // transfer some fees of amount to market creator
            let external_fees = T::ExternalFees::distribute(market_id, base_asset, &who, amount);
            let amount_minus_fees =
                amount.checked_sub(&external_fees).ok_or(Error::<T>::Unexpected)?;
            let pot_account = Self::pot_account(market_id);

            T::AssetManager::transfer(market.base_asset, &who, &pot_account, amount_minus_fees)?;
            T::AssetManager::deposit(asset, &who, amount_minus_fees)?;

            let mut total_reward =
                TotalRewards::<T>::get(market_id).unwrap_or(BalanceOf::<T>::zero());
            total_reward = total_reward.saturating_add(amount_minus_fees);
            TotalRewards::<T>::insert(market_id, total_reward);

            Self::deposit_event(Event::OutcomeBought {
                market_id,
                buyer: who,
                asset,
                amount_minus_fees,
                fees: external_fees,
            });

            Ok(())
        }

        /// Claim winnings from a resolved market.
        ///
        /// Complexity: `O(1)`
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::claim_rewards())]
        #[frame_support::transactional]
        pub fn claim_rewards(origin: OriginFor<T>, market_id: MarketIdOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.status == MarketStatus::Resolved, Error::<T>::MarketIsNotResolvedYet);
            ensure!(market.scoring_rule == ScoringRule::Parimutuel, Error::<T>::InvalidScoringRule);
            ensure!(
                matches!(market.market_type, MarketType::Categorical(_)),
                Error::<T>::OnlyCategoricalMarketsAllowed
            );
            let winning_outcome = market.resolved_outcome.ok_or(Error::<T>::NoResolvedOutcome)?;
            let pot_account = Self::pot_account(market_id);
            let (winning_asset, payoff, slashable_asset_balance) = match winning_outcome {
                OutcomeReport::Categorical(category_index) => {
                    let winning_asset = Asset::ParimutuelShare(market_id, category_index);
                    // each Parimutuel outcome asset has the market id included
                    // this allows us to query all outstanding shares for each discrete asset
                    let outcome_total = T::AssetManager::total_issuance(winning_asset);
                    // use refund extrinsic in case there is no winner
                    ensure!(outcome_total != <BalanceOf<T>>::zero(), Error::<T>::NoWinner);
                    let winning_balance = T::AssetManager::free_balance(winning_asset, &who);
                    ensure!(!winning_balance.is_zero(), Error::<T>::NoWinningShares);
                    debug_assert!(
                        outcome_total >= winning_balance,
                        "The outcome issuance should be at least as high as the individual \
                         balance of this outcome!"
                    );

                    let pot_total = T::AssetManager::free_balance(market.base_asset, &pot_account);
                    // use bdiv, because pot_total / outcome_total could be
                    // a rational number imprecisely rounded to the next integer
                    // however we need the precision here to calculate the correct payout
                    // by bdiv we multiply it with BASE and using bmul we divide it by BASE again
                    // Fugayzi, fugazi. It's a whazy. It's a woozie. It's fairy dust.
                    debug_assert!(
                        outcome_total != <BalanceOf<T>>::zero(),
                        "If winning balance is non-zero, then the outcome total can only be at \
                         least as high as the winning balance (non-zero too)!"
                    );
                    let payoff_ratio_mul_base: BalanceOf<T> =
                        bdiv(pot_total.saturated_into(), outcome_total.saturated_into())?
                            .saturated_into();
                    let payoff: BalanceOf<T> = bmul(
                        payoff_ratio_mul_base.saturated_into(),
                        winning_balance.saturated_into(),
                    )?
                    .saturated_into();

                    Self::check_values(
                        winning_balance,
                        pot_total,
                        outcome_total,
                        payoff_ratio_mul_base,
                        payoff,
                    )?;

                    let slashable_asset_balance = winning_balance;

                    (winning_asset, payoff, slashable_asset_balance)
                }
                OutcomeReport::Scalar(_) => {
                    return Err(Error::<T>::ScalarMarketsNotAllowed.into());
                }
            };

            T::AssetManager::slash(winning_asset, &who, slashable_asset_balance);

            let remaining_bal = T::AssetManager::free_balance(market.base_asset, &pot_account);
            let actual_payoff = payoff.min(remaining_bal);

            T::AssetManager::transfer(market.base_asset, &pot_account, &who, actual_payoff)?;

            Self::deposit_event(Event::RewardsClaimed {
                market_id,
                asset: winning_asset,
                balance: slashable_asset_balance,
                actual_payoff,
                sender: who.clone(),
            });

            Ok(())
        }

        /// Refund the collateral of losing categorical outcome assets
        /// in case that there was no account betting on the winner outcome.
        ///
        /// # Arguments
        ///
        /// - `refund_asset`: The outcome asset to refund.
        ///
        /// Complexity: `O(log(n))``, where `n` is the number of categorical assets the market can have.
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::refund_pot())]
        #[frame_support::transactional]
        pub fn refund_pot(origin: OriginFor<T>, refund_asset: AssetOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let market_id = match refund_asset {
                Asset::ParimutuelShare(market_id, _) => market_id,
                _ => return Err(Error::<T>::NotParimutuelOutcome.into()),
            };
            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.status == MarketStatus::Resolved, Error::<T>::MarketIsNotResolvedYet);
            ensure!(market.scoring_rule == ScoringRule::Parimutuel, Error::<T>::InvalidScoringRule);
            ensure!(
                matches!(market.market_type, MarketType::Categorical(_)),
                Error::<T>::OnlyCategoricalMarketsAllowed
            );
            let market_assets = Self::outcome_assets(market_id, &market)?;
            ensure!(
                market_assets.binary_search(&refund_asset).is_ok(),
                Error::<T>::InvalidOutcomeAsset
            );
            let winning_outcome = market.resolved_outcome.ok_or(Error::<T>::NoResolvedOutcome)?;
            let pot_account = Self::pot_account(market_id);
            let (refund_asset, refund_balance) = match winning_outcome {
                OutcomeReport::Categorical(category_index) => {
                    let winning_asset = Asset::ParimutuelShare(market_id, category_index);
                    let outcome_total = T::AssetManager::total_issuance(winning_asset);
                    ensure!(outcome_total == <BalanceOf<T>>::zero(), Error::<T>::RefundNotAllowed);

                    let refund_balance = T::AssetManager::free_balance(refund_asset, &who);
                    ensure!(!refund_balance.is_zero(), Error::<T>::RefundableBalanceIsZero);
                    debug_assert!(
                        refund_asset != winning_asset,
                        "Since we were checking the total issuance of the winning asset to be \
                         zero, if the refund balance is non-zero, then the winning asset can't be \
                         the refund asset!"
                    );

                    (refund_asset, refund_balance)
                }
                OutcomeReport::Scalar(_) => return Err(Error::<T>::ScalarMarketsNotAllowed.into()),
            };

            let slashable_asset_balance = refund_balance;
            T::AssetManager::slash(refund_asset, &who, slashable_asset_balance);

            let pot_total = T::AssetManager::free_balance(market.base_asset, &pot_account);
            debug_assert!(
                pot_total >= refund_balance,
                "The pot total should be at least as high as the individual refund balance!"
            );
            let refund_balance = refund_balance.min(pot_total);

            T::AssetManager::transfer(market.base_asset, &pot_account, &who, refund_balance)?;

            Self::deposit_event(Event::BalanceRefunded {
                market_id,
                asset: refund_asset,
                refunded_balance: refund_balance,
                sender: who.clone(),
            });

            Ok(())
        }
    }

    impl<T> Pallet<T>
    where
        T: Config,
    {
        #[inline]
        pub(crate) fn pot_account(market_id: MarketIdOf<T>) -> AccountIdOf<T> {
            T::PalletId::get().into_sub_account_truncating(market_id)
        }

        /// Check the values for validity.
        fn check_values(
            winning_balance: BalanceOf<T>,
            pot_total: BalanceOf<T>,
            outcome_total: BalanceOf<T>,
            payoff_ratio_mul_base: BalanceOf<T>,
            payoff: BalanceOf<T>,
        ) -> DispatchResult {
            ensure!(pot_total >= winning_balance, Error::<T>::InsufficientFundsInPotAccount);
            ensure!(pot_total >= outcome_total, Error::<T>::OutcomeIssuanceGreaterCollateral);
            debug_assert!(
                payoff_ratio_mul_base >= BASE.saturated_into(),
                "The payoff ratio should be greater than or equal to BASE!"
            );
            debug_assert!(
                payoff >= winning_balance,
                "The payoff in collateral should be greater than or equal to the winning outcome \
                 balance."
            );
            debug_assert!(
                pot_total >= payoff,
                "The payoff in collateral should not exceed the total amount of collateral!"
            );
            Ok(())
        }

        pub fn outcome_assets(
            market_id: MarketIdOf<T>,
            market: &MarketOf<T>,
        ) -> Result<Vec<AssetOf<T>>, DispatchError> {
            match market.market_type {
                MarketType::Categorical(categories) => {
                    let mut assets = Vec::new();
                    for i in 0..categories {
                        assets.push(Asset::ParimutuelShare(market_id, i));
                    }
                    Ok(assets)
                }
                MarketType::Scalar(_) => Err(Error::<T>::ScalarMarketsNotAllowed.into()),
            }
        }
    }
}

#[cfg(any(feature = "runtime-benchmarks", test))]
pub(crate) fn market_mock<T>() -> MarketOf<T>
where
    T: crate::Config,
{
    use frame_support::traits::Get;
    use sp_runtime::{traits::AccountIdConversion, Perbill};
    use zeitgeist_primitives::types::{
        Asset, Deadlines, MarketBonds, MarketCreation, MarketDisputeMechanism, MarketPeriod,
        MarketStatus, MarketType, ScoringRule,
    };

    zeitgeist_primitives::types::Market {
        base_asset: Asset::Ztg,
        creation: MarketCreation::Permissionless,
        creator_fee: Perbill::zero(),
        creator: T::PalletId::get().into_account_truncating(),
        market_type: MarketType::Scalar(0..=100),
        dispute_mechanism: Some(MarketDisputeMechanism::Authorized),
        metadata: Default::default(),
        oracle: T::PalletId::get().into_account_truncating(),
        period: MarketPeriod::Block(Default::default()),
        deadlines: Deadlines {
            grace_period: 1_u32.into(),
            oracle_duration: 1_u32.into(),
            dispute_duration: 1_u32.into(),
        },
        report: None,
        resolved_outcome: None,
        scoring_rule: ScoringRule::Parimutuel,
        status: MarketStatus::Active,
        bonds: MarketBonds::default(),
    }
}
