// Copyright 2023-2024 Forecasting Technologies LTD.
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

mod benchmarking;
mod mock;
mod tests;
mod utils;
pub mod weights;

pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::weights::WeightInfoZeitgeist;
    use core::marker::PhantomData;
    use frame_support::{
        ensure, log,
        pallet_prelude::{Decode, DispatchError, Encode, TypeInfo},
        require_transactional,
        traits::{fungibles::{Inspect, Create}, Get, IsType, StorageVersion},
        PalletId, RuntimeDebug,
    };
    use frame_system::{
        ensure_signed,
        pallet_prelude::{BlockNumberFor, OriginFor},
    };
    use orml_traits::MultiCurrency;
    use pallet_assets::ManagedDestroy;
    use sp_runtime::{
        traits::{AccountIdConversion, CheckedSub, Zero},
        DispatchResult,
    };
    use zeitgeist_primitives::{
        math::fixed::FixedMulDiv,
        traits::DistributeFees,
        types::{Asset, BaseAsset, Market, MarketStatus, MarketType, OutcomeReport, ScoringRule},
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The module handling the creation of market assets.
        type AssetCreator: Create<Self::AccountId, AssetId = AssetOf<Self>, Balance = BalanceOf<Self>>;

        /// The module handling the destruction of market assets.
        type AssetDestroyer: ManagedDestroy<Self::AccountId, AssetId = AssetOf<Self>, Balance = BalanceOf<Self>>;

        /// The api to handle different asset classes.
        type AssetManager: MultiCurrency<Self::AccountId, CurrencyId = AssetOf<Self>>;

        /// The way how fees are taken from the market base asset.
        type ExternalFees: DistributeFees<
                Asset = Asset<MarketIdOf<Self>>,
                AccountId = AccountIdOf<Self>,
                Balance = BalanceOf<Self>,
                MarketId = MarketIdOf<Self>,
            >;

        type MarketCommons: MarketCommonsPalletApi<
                AccountId = Self::AccountId,
                BlockNumber = Self::BlockNumber,
                Balance = BalanceOf<Self>,
            >;

        /// The minimum amount each bet must be. Must be larger than or equal to the existential
        /// deposit of parimutuel shares.
        #[pallet::constant]
        type MinBetSize: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Weights generated by benchmarks.
        type WeightInfo: WeightInfoZeitgeist;
    }

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);
    const LOG_TARGET: &str = "runtime::zrml-parimutuel";

    pub(crate) type AssetOf<T> = Asset<MarketIdOf<T>>;
    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type BalanceOf<T> =
        <<T as Config>::AssetManager as MultiCurrency<AccountIdOf<T>>>::Balance;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub(crate) type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;
    pub(crate) type MarketOf<T> =
        Market<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>, MomentOf<T>, BaseAsset>;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

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
            withdrawn_asset_balance: BalanceOf<T>,
            base_asset_payoff: BalanceOf<T>,
            sender: AccountIdOf<T>,
        },
        /// A market base asset was refunded.
        BalanceRefunded {
            market_id: MarketIdOf<T>,
            asset: AssetOf<T>,
            refunded_balance: BalanceOf<T>,
            sender: AccountIdOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// There was no buyer for the winning outcome or all winners already claimed their rewards.
        /// Use the `refund` extrinsic to get the initial bet back,
        /// in case there was no buyer for the winning outcome.
        NoRewardShareOutstanding,
        /// The market is not active.
        MarketIsNotActive,
        /// The specified amount is below the minimum bet size.
        AmountBelowMinimumBetSize,
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
        /// Only categorical markets are allowed for parimutuels.
        NotCategorical,
        /// There is no reward to distribute.
        NoRewardToDistribute,
        /// Action cannot be completed because an unexpected error has occurred. This should be
        /// reported to protocol maintainers.
        InconsistentState(InconsistentStateError),
    }

    // NOTE: these errors should never happen.
    #[derive(Encode, Decode, Eq, PartialEq, TypeInfo, frame_support::PalletError, RuntimeDebug)]
    pub enum InconsistentStateError {
        /// There are not enough funds in the pot to reward the calculated amount.
        InsufficientFundsInPotAccount,
        /// The outcome issuance is greater than the market base asset.
        OutcomeIssuanceGreaterCollateral,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Buy parimutuel shares for the market's base asset.
        ///
        /// # Arguments
        ///
        /// - `asset`: The outcome asset to buy the shares of.
        /// - `amount`: The amount of base asset to spend
        /// and of parimutuel shares to receive.
        /// Keep in mind that there are external fees taken from this amount.
        ///
        /// Complexity: `O(1)`
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::buy())]
        #[frame_support::transactional]
        pub fn buy(
            origin: OriginFor<T>,
            asset: Asset<MarketIdOf<T>>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::do_buy(who, asset, amount)?;

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

            Self::do_claim_rewards(who, market_id)?;

            Ok(())
        }

        /// Refund the base asset of losing categorical outcome assets
        /// in case that there was no account betting on the winner outcome.
        ///
        /// # Arguments
        ///
        /// - `refund_asset`: The outcome asset to refund.
        ///
        /// Complexity: `O(1)`
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::claim_refunds())]
        #[frame_support::transactional]
        pub fn claim_refunds(origin: OriginFor<T>, refund_asset: AssetOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::do_claim_refunds(who, refund_asset)?;

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
            payoff: BalanceOf<T>,
        ) -> DispatchResult {
            ensure!(
                pot_total >= winning_balance,
                Error::<T>::InconsistentState(
                    InconsistentStateError::InsufficientFundsInPotAccount
                )
            );
            ensure!(
                pot_total >= outcome_total,
                Error::<T>::InconsistentState(
                    InconsistentStateError::OutcomeIssuanceGreaterCollateral
                )
            );
            if payoff < winning_balance {
                log::debug!(
                    target: LOG_TARGET,
                    "The payoff in base asset should be greater than or equal to the winning outcome \
                    balance."
                );
                debug_assert!(false);
            }
            if pot_total < payoff {
                log::debug!(
                    target: LOG_TARGET,
                    "The payoff in base asset should not exceed the total amount of the base asset!"
                );
                debug_assert!(false);
            }
            Ok(())
        }

        pub fn market_assets_contains(market: &MarketOf<T>, asset: &AssetOf<T>) -> DispatchResult {
            if let Asset::ParimutuelShare(_, i) = asset {
                match market.market_type {
                    MarketType::Categorical(categories) => {
                        ensure!(*i < categories, Error::<T>::InvalidOutcomeAsset);
                        return Ok(());
                    }
                    MarketType::Scalar(_) => return Err(Error::<T>::NotCategorical.into()),
                }
            }
            Err(Error::<T>::NotParimutuelOutcome.into())
        }

        #[require_transactional]
        fn do_buy(who: T::AccountId, asset: AssetOf<T>, amount: BalanceOf<T>) -> DispatchResult {
            let market_id = match asset {
                Asset::ParimutuelShare(market_id, _) => market_id,
                _ => return Err(Error::<T>::NotParimutuelOutcome.into()),
            };
            let market = T::MarketCommons::market(&market_id)?;
            let base_asset = market.base_asset;
            ensure!(
                T::AssetManager::ensure_can_withdraw(base_asset.into(), &who, amount).is_ok(),
                Error::<T>::InsufficientBalance
            );
            ensure!(market.status == MarketStatus::Active, Error::<T>::MarketIsNotActive);
            ensure!(market.scoring_rule == ScoringRule::Parimutuel, Error::<T>::InvalidScoringRule);
            ensure!(
                matches!(market.market_type, MarketType::Categorical(_)),
                Error::<T>::NotCategorical
            );
            Self::market_assets_contains(&market, &asset)?;
            let pot_account = Self::pot_account(market_id);

            if !T::AssetCreator::asset_exists(asset) {
                let admin = pot_account.clone();
                let is_sufficient = true;
                let min_balance = 1u8;
                T::AssetCreator::create(asset, admin, is_sufficient, min_balance.into())?;
            }

            let external_fees =
                T::ExternalFees::distribute(market_id, base_asset.into(), &who, amount);
            let amount_minus_fees =
                amount.checked_sub(&external_fees).ok_or(Error::<T>::Unexpected)?;
            ensure!(
                amount_minus_fees >= T::MinBetSize::get(),
                Error::<T>::AmountBelowMinimumBetSize
            );

            T::AssetManager::transfer(base_asset.into(), &who, &pot_account, amount_minus_fees)?;
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

        fn ensure_parimutuel_market_resolved(market: &MarketOf<T>) -> DispatchResult {
            ensure!(market.status == MarketStatus::Resolved, Error::<T>::MarketIsNotResolvedYet);
            ensure!(market.scoring_rule == ScoringRule::Parimutuel, Error::<T>::InvalidScoringRule);
            ensure!(
                matches!(market.market_type, MarketType::Categorical(_)),
                Error::<T>::NotCategorical
            );
            Ok(())
        }

        fn get_winning_asset(
            market_id: MarketIdOf<T>,
            market: &MarketOf<T>,
        ) -> Result<AssetOf<T>, DispatchError> {
            let winning_outcome =
                market.resolved_outcome.clone().ok_or(Error::<T>::NoResolvedOutcome)?;
            let winning_asset = match winning_outcome {
                OutcomeReport::Categorical(category_index) => {
                    Asset::ParimutuelShare(market_id, category_index)
                }
                OutcomeReport::Scalar(_) => return Err(Error::<T>::NotCategorical.into()),
            };
            Ok(winning_asset)
        }

        #[require_transactional]
        fn do_claim_rewards(who: T::AccountId, market_id: MarketIdOf<T>) -> DispatchResult {
            let market = T::MarketCommons::market(&market_id)?;
            Self::ensure_parimutuel_market_resolved(&market)?;
            let winning_asset = Self::get_winning_asset(market_id, &market)?;
            // each Parimutuel outcome asset has the market id included
            // this allows us to query all outstanding shares for each discrete asset
            let outcome_total = T::AssetManager::total_issuance(winning_asset);
            // if there are no outstanding reward shares, but the pot account is not empty
            // then use the refund extrinsic to get the initial bet back
            ensure!(outcome_total != BalanceOf::<T>::zero(), Error::<T>::NoRewardShareOutstanding);
            let winning_balance = T::AssetManager::free_balance(winning_asset, &who);
            ensure!(!winning_balance.is_zero(), Error::<T>::NoWinningShares);
            if outcome_total < winning_balance {
                log::debug!(
                    target: LOG_TARGET,
                    "The outcome issuance should be at least as high as the individual balance of \
                     this outcome!"
                );
                debug_assert!(false);
            }

            let pot_account = Self::pot_account(market_id);
            let pot_total = T::AssetManager::free_balance(market.base_asset.into(), &pot_account);
            let payoff = pot_total.bmul_bdiv(winning_balance, outcome_total)?;

            Self::check_values(winning_balance, pot_total, outcome_total, payoff)?;

            let withdrawn_asset_balance = winning_balance;

            T::AssetManager::withdraw(winning_asset, &who, withdrawn_asset_balance)?;

            let remaining_bal =
                T::AssetManager::free_balance(market.base_asset.into(), &pot_account);
            let base_asset_payoff = payoff.min(remaining_bal);

            T::AssetManager::transfer(
                market.base_asset.into(),
                &pot_account,
                &who,
                base_asset_payoff,
            )?;

            Self::deposit_event(Event::RewardsClaimed {
                market_id,
                asset: winning_asset,
                withdrawn_asset_balance,
                base_asset_payoff,
                sender: who.clone(),
            });

            Ok(())
        }

        #[require_transactional]
        fn do_claim_refunds(who: T::AccountId, refund_asset: AssetOf<T>) -> DispatchResult {
            let market_id = match refund_asset {
                Asset::ParimutuelShare(market_id, _) => market_id,
                _ => return Err(Error::<T>::NotParimutuelOutcome.into()),
            };
            let market = T::MarketCommons::market(&market_id)?;
            Self::ensure_parimutuel_market_resolved(&market)?;
            Self::market_assets_contains(&market, &refund_asset)?;
            let winning_asset = Self::get_winning_asset(market_id, &market)?;
            let outcome_total = T::AssetManager::total_issuance(winning_asset);
            ensure!(outcome_total == <BalanceOf<T>>::zero(), Error::<T>::RefundNotAllowed);

            let refund_balance = T::AssetManager::free_balance(refund_asset, &who);
            ensure!(!refund_balance.is_zero(), Error::<T>::RefundableBalanceIsZero);
            if refund_asset == winning_asset {
                log::debug!(
                    target: LOG_TARGET,
                    "Since we were checking the total issuance of the winning asset to be zero, if \
                     the refund balance is non-zero, then the winning asset can't be the refund \
                     asset!"
                );
                debug_assert!(false);
            }

            T::AssetManager::withdraw(refund_asset, &who, refund_balance)?;

            let pot_account = Self::pot_account(market_id);
            let pot_total = T::AssetManager::free_balance(market.base_asset.into(), &pot_account);
            if pot_total < refund_balance {
                log::debug!(
                    target: LOG_TARGET,
                    "The pot total is lower than the refund balance! This should never happen!"
                );
                debug_assert!(false);
            }
            let refund_balance = refund_balance.min(pot_total);

            T::AssetManager::transfer(
                market.base_asset.into(),
                &pot_account,
                &who,
                refund_balance,
            )?;

            Self::deposit_event(Event::BalanceRefunded {
                market_id,
                asset: refund_asset,
                refunded_balance: refund_balance,
                sender: who.clone(),
            });

            Ok(())
        }
    }
}
