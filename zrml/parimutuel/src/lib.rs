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
pub mod types;
pub mod weights;

pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::weights::WeightInfoZeitgeist;
    use core::{cmp::Ordering, marker::PhantomData};
    use frame_support::{
        ensure,
        traits::{Get, IsType, StorageVersion},
        PalletId,
    };
    use frame_system::{
        ensure_signed,
        pallet_prelude::{BlockNumberFor, OriginFor},
    };
    use orml_traits::MultiCurrency;
    use sp_runtime::{
        traits::{AccountIdConversion, CheckedSub, Zero},
        DispatchError, DispatchResult, Perquintill, SaturatedConversion, Saturating,
    };
    use zeitgeist_primitives::{
        constants::BASE,
        math::fixed::*,
        traits::DistributeFees,
        types::{
            Asset, Market, MarketStatus, MarketType, Outcome, OutcomeReport, ScalarPosition,
            ScoringRule,
        },
    };
    use zrml_market_commons::MarketCommonsPalletApi;

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

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Buy parimutuel shares.
        ///
        /// # Arguments
        ///
        /// - `asset`: The outcome asset to buy the shares of.
        /// - `amount`: The amount of collateral (base asset) to spend
        /// and of parimutuel shares to receive.
        /// Keep in mind that market creator fees are taken from this amount.
        #[pallet::call_index(0)]
        #[pallet::weight(5000)]
        #[frame_support::transactional]
        pub fn buy(
            origin: OriginFor<T>,
            asset: Asset<MarketIdOf<T>>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(amount >= T::MinBetSize::get(), Error::<T>::AmountTooSmall);

            let outcome = match asset {
                Asset::ParimutuelShare(outcome) => outcome,
                _ => return Err(Error::<T>::NotParimutuelOutcome.into()),
            };
            let market_id = match outcome {
                Outcome::CategoricalOutcome(market_id, _) => market_id,
                Outcome::ScalarOutcome(market_id, _) => market_id,
            };
            let market = T::MarketCommons::market(&market_id)?;
            let base_asset = market.base_asset;
            ensure!(
                T::AssetManager::ensure_can_withdraw(base_asset, &who, amount).is_ok(),
                Error::<T>::InsufficientBalance
            );
            ensure!(market.status == MarketStatus::Active, Error::<T>::MarketIsNotActive);
            ensure!(market.scoring_rule == ScoringRule::Parimutuel, Error::<T>::InvalidScoringRule);
            let market_assets = Self::outcome_assets(market_id, &market);
            ensure!(market_assets.binary_search(&asset).is_ok(), Error::<T>::InvalidOutcomeAsset);

            // transfer some fees of amount to market creator
            let external_fees = T::ExternalFees::distribute(market_id, base_asset, &who, amount);
            let amount_minus_fees =
                amount.checked_sub(&external_fees).ok_or(Error::<T>::Unexpected)?;
            let pot_account = Self::pot_account(market_id);

            T::AssetManager::transfer(market.base_asset, &who, &pot_account, amount_minus_fees)?;
            T::AssetManager::deposit(asset, &who, amount_minus_fees)?;

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
        #[pallet::call_index(1)]
        #[pallet::weight(5000)]
        #[frame_support::transactional]
        pub fn claim_rewards(origin: OriginFor<T>, market_id: MarketIdOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.status == MarketStatus::Resolved, Error::<T>::MarketIsNotResolvedYet);
            ensure!(market.scoring_rule == ScoringRule::Parimutuel, Error::<T>::InvalidScoringRule);
            let winning_outcome = market.resolved_outcome.ok_or(Error::<T>::NoResolvedOutcome)?;
            let pot_account = Self::pot_account(market_id);
            let winning_assets = match winning_outcome {
                OutcomeReport::Categorical(category_index) => {
                    let winning_asset = Asset::ParimutuelShare(Outcome::CategoricalOutcome(
                        market_id,
                        category_index,
                    ));
                    // each Parimutuel outcome asset has the market id included
                    // this allows us to query all outstanding shares for each discrete asset
                    let outcome_total = T::AssetManager::total_issuance(winning_asset);
                    // use refund extrinsic in case there is no winner
                    ensure!(outcome_total != BalanceOf::<T>::zero(), Error::<T>::NoWinner);
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
                        outcome_total != BalanceOf::<T>::zero(),
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

                    vec![(winning_asset, payoff, slashable_asset_balance)]
                }
                OutcomeReport::Scalar(value) => {
                    let long_asset = Asset::ParimutuelShare(Outcome::ScalarOutcome(
                        market_id,
                        ScalarPosition::Long,
                    ));
                    let short_asset = Asset::ParimutuelShare(Outcome::ScalarOutcome(
                        market_id,
                        ScalarPosition::Short,
                    ));

                    let long_balance = T::AssetManager::free_balance(long_asset, &who);
                    let short_balance = T::AssetManager::free_balance(short_asset, &who);
                    ensure!(
                        !long_balance.is_zero() || !short_balance.is_zero(),
                        Error::<T>::NoWinningShares
                    );

                    let bound = if let MarketType::Scalar(range) = market.market_type {
                        range
                    } else {
                        return Err(Error::<T>::InvalidMarketType.into());
                    };

                    let (long_percent, short_percent) =
                        Self::calc_final_payoff_percentages(value, *bound.start(), *bound.end());

                    let pot_total = T::AssetManager::free_balance(market.base_asset, &pot_account);

                    let payoff_long_portion =
                        Self::get_payoff(long_asset, long_percent, pot_total, long_balance)?;

                    let payoff_short_portion: BalanceOf<T> =
                        Self::get_payoff(short_asset, short_percent, pot_total, short_balance)?;

                    // Ensure the pot account has enough to pay out - if this is
                    // ever not true then we have an accounting problem.
                    ensure!(
                        pot_total >= payoff_long_portion.saturating_add(payoff_short_portion),
                        Error::<T>::InsufficientFundsInPotAccount,
                    );

                    let slashable_long_balance = long_balance;
                    let slashable_short_balance = short_balance;

                    vec![
                        (long_asset, payoff_long_portion, slashable_long_balance),
                        (short_asset, payoff_short_portion, slashable_short_balance),
                    ]
                }
            };

            for (asset, payoff, slashable_asset_balance) in winning_assets {
                T::AssetManager::slash(asset, &who, slashable_asset_balance);

                let remaining_bal = T::AssetManager::free_balance(market.base_asset, &pot_account);
                let actual_payoff = payoff.min(remaining_bal);

                T::AssetManager::transfer(market.base_asset, &pot_account, &who, actual_payoff)?;
                // The if-check prevents scalar markets to emit events even if sender only owns one
                // of the outcome tokens.
                if slashable_asset_balance != BalanceOf::<T>::zero() {
                    Self::deposit_event(Event::RewardsClaimed {
                        market_id,
                        asset,
                        balance: slashable_asset_balance,
                        actual_payoff,
                        sender: who.clone(),
                    });
                }
            }

            Ok(())
        }

        /// Refund the collateral of losing categorical outcome assets
        /// in case that there was no account betting on the winner outcome.
        #[pallet::call_index(2)]
        #[pallet::weight(5000)]
        #[frame_support::transactional]
        pub fn refund_pot(origin: OriginFor<T>, refund_asset: AssetOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let outcome = match refund_asset {
                Asset::ParimutuelShare(outcome) => outcome,
                _ => return Err(Error::<T>::NotParimutuelOutcome.into()),
            };
            let market_id = match outcome {
                Outcome::CategoricalOutcome(market_id, _) => market_id,
                Outcome::ScalarOutcome(market_id, _) => market_id,
            };
            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.status == MarketStatus::Resolved, Error::<T>::MarketIsNotResolvedYet);
            ensure!(market.scoring_rule == ScoringRule::Parimutuel, Error::<T>::InvalidScoringRule);
            let market_assets = Self::outcome_assets(market_id, &market);
            ensure!(
                market_assets.binary_search(&refund_asset).is_ok(),
                Error::<T>::InvalidOutcomeAsset
            );
            let winning_outcome = market.resolved_outcome.ok_or(Error::<T>::NoResolvedOutcome)?;
            let pot_account = Self::pot_account(market_id);
            let (refund_asset, refund_balance) = match winning_outcome {
                OutcomeReport::Categorical(category_index) => {
                    let winning_asset = Asset::ParimutuelShare(Outcome::CategoricalOutcome(
                        market_id,
                        category_index,
                    ));
                    let outcome_total = T::AssetManager::total_issuance(winning_asset);
                    ensure!(outcome_total == BalanceOf::<T>::zero(), Error::<T>::RefundNotAllowed);

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
                OutcomeReport::Scalar(_) => return Err(Error::<T>::NoCategoricalOutcome.into()),
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

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type ExternalFees: DistributeFees<
                Asset = Asset<MarketIdOf<Self>>,
                AccountId = AccountIdOf<Self>,
                Balance = BalanceOf<Self>,
                MarketId = MarketIdOf<Self>,
            >;

        /// Event
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type MarketCommons: MarketCommonsPalletApi<
                AccountId = Self::AccountId,
                BlockNumber = Self::BlockNumber,
                Balance = BalanceOf<Self>,
            >;

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

    #[pallet::error]
    pub enum Error<T> {
        // There was no buyer of the winning outcome.
        // Use the `refund` extrinsic to get the initial bet back.
        NoWinner,
        OutcomeMismatch,
        MarketIsNotActive,
        AmountTooSmall,
        NotParimutuelOutcome,
        InvalidOutcomeAsset,
        InvalidScoringRule,
        InsufficientBalance,
        MarketIsNotResolvedYet,
        Unexpected,
        NoResolvedOutcome,
        RefundNotAllowed,
        RefundableBalanceIsZero,
        NoWinningShares,
        InsufficientFundsInPotAccount,
        InvalidMarketType,
        OutcomeIssuanceGreaterCollateral,
        NoCategoricalOutcome,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        OutcomeBought {
            market_id: MarketIdOf<T>,
            buyer: AccountIdOf<T>,
            asset: AssetOf<T>,
            amount_minus_fees: BalanceOf<T>,
            fees: BalanceOf<T>,
        },
        RewardsClaimed {
            market_id: MarketIdOf<T>,
            asset: AssetOf<T>,
            balance: BalanceOf<T>,
            actual_payoff: BalanceOf<T>,
            sender: AccountIdOf<T>,
        },
        BalanceRefunded {
            market_id: MarketIdOf<T>,
            asset: AssetOf<T>,
            refunded_balance: BalanceOf<T>,
            sender: AccountIdOf<T>,
        },
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    impl<T> Pallet<T>
    where
        T: Config,
    {
        #[inline]
        fn pot_account(market_id: MarketIdOf<T>) -> AccountIdOf<T> {
            T::PalletId::get().into_sub_account_truncating(market_id)
        }

        fn get_payoff(
            asset: AssetOf<T>,
            final_payoff_percentage: Perquintill,
            pot_total: BalanceOf<T>,
            outcome_balance: BalanceOf<T>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            if outcome_balance.is_zero() {
                return Ok(BalanceOf::<T>::zero());
            }
            let max_payoff = final_payoff_percentage.mul_floor(pot_total.saturated_into::<u128>());
            let current_total_asset_amount = T::AssetManager::total_issuance(asset);
            debug_assert!(
                outcome_balance <= current_total_asset_amount,
                "Outcome asset amount should be less than or equal to total issuance!"
            );
            debug_assert!(
                current_total_asset_amount > BalanceOf::<T>::zero(),
                "The total issuance of the outcome asset should be greater than zero, because the \
                 individual outcome balance is ensured to be non-zero above!"
            );
            let payoff_ratio_mul_base: BalanceOf<T> =
                bdiv(max_payoff.saturated_into(), current_total_asset_amount.saturated_into())?
                    .saturated_into();
            let payoff_portion: BalanceOf<T> =
                bmul(payoff_ratio_mul_base.saturated_into(), outcome_balance.saturated_into())?
                    .saturated_into();

            debug_assert!({
                let max_payoff = max_payoff.saturated_into::<BalanceOf<T>>();
                match max_payoff.cmp(&current_total_asset_amount) {
                    Ordering::Greater => payoff_portion > outcome_balance, // gain profit, low cost `outcome_balance`
                    Ordering::Less => payoff_portion < outcome_balance, // loss, high cost `outcome_balance`
                    Ordering::Equal => payoff_portion == outcome_balance, // no profit, cost equal to payoff
                }
            });
            debug_assert!(
                payoff_portion <= max_payoff.saturated_into::<BalanceOf<T>>(),
                "Individual payoff should only be a portion of the maximum payoff!"
            );

            Ok(payoff_portion)
        }

        fn calc_final_payoff_percentages(
            final_value: u128,
            low: u128,
            high: u128,
        ) -> (Perquintill, Perquintill) {
            if final_value <= low {
                return (Perquintill::zero(), Perquintill::one());
            }
            if final_value >= high {
                return (Perquintill::one(), Perquintill::zero());
            }

            let payoff_long: Perquintill = Perquintill::from_rational(
                final_value.saturating_sub(low),
                high.saturating_sub(low),
            );
            let payoff_short: Perquintill = Perquintill::from_parts(
                Perquintill::one().deconstruct().saturating_sub(payoff_long.deconstruct()),
            );
            (payoff_long, payoff_short)
        }

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

        pub fn outcome_assets(market_id: MarketIdOf<T>, market: &MarketOf<T>) -> Vec<AssetOf<T>> {
            match market.market_type {
                MarketType::Categorical(categories) => {
                    let mut assets = Vec::new();
                    for i in 0..categories {
                        assets.push(Asset::ParimutuelShare(Outcome::CategoricalOutcome(
                            market_id, i,
                        )));
                    }
                    assets
                }
                MarketType::Scalar(_) => {
                    vec![
                        Asset::ParimutuelShare(Outcome::ScalarOutcome(
                            market_id,
                            ScalarPosition::Long,
                        )),
                        Asset::ParimutuelShare(Outcome::ScalarOutcome(
                            market_id,
                            ScalarPosition::Short,
                        )),
                    ]
                }
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
