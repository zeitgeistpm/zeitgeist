//! # Simple disputes
//!
//! Manages market disputes and resolutions.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod simple_disputes_pallet_api;

pub use pallet::*;
pub use simple_disputes_pallet_api::SimpleDisputesPalletApi;

#[frame_support::pallet]
mod pallet {
    use crate::SimpleDisputesPalletApi;
    use alloc::vec::Vec;
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::DispatchResult,
        traits::{Currency, Get, Hooks, Imbalance, IsType, ReservableCurrency},
        PalletId,
    };
    use sp_runtime::{traits::CheckedDiv, ArithmeticError, DispatchError, SaturatedConversion};
    use zeitgeist_primitives::{
        traits::{DisputeApi, Swaps, ZeitgeistMultiReservableCurrency},
        types::{Asset, Market, MarketDispute, MarketStatus, OutcomeReport},
    };
    use zrml_liquidity_mining::LiquidityMiningPalletApi;
    use zrml_market_commons::MarketCommonsPalletApi;

    type BalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type CurrencyOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Currency;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub(crate) type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;
    type NegativeImbalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Common market parameters
        type LiquidityMining: LiquidityMiningPalletApi<
            AccountId = Self::AccountId,
            Balance = BalanceOf<Self>,
            BlockNumber = Self::BlockNumber,
            MarketId = MarketIdOf<Self>,
        >;

        /// The identifier of individual markets.
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;

        /// The base amount of currency that must be bonded to ensure the oracle reports
        ///  in a timely manner.
        type OracleBond: Get<BalanceOf<Self>>;

        /// The pallet identifier.
        type PalletId: Get<PalletId>;

        /// Swap shares
        type Shares: ZeitgeistMultiReservableCurrency<
            Self::AccountId,
            Balance = BalanceOf<Self>,
            CurrencyId = Asset<<Self::MarketCommons as MarketCommonsPalletApi>::MarketId>,
        >;

        /// Swaps pallet
        type Swaps: Swaps<Self::AccountId, Balance = BalanceOf<Self>, MarketId = MarketIdOf<Self>>;

        /// The base amount of currency that must be bonded for a permissionless market,
        /// guaranteeing that it will resolve as anything but `Invalid`.
        type ValidityBond: Get<BalanceOf<Self>>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 1. Any resolution must either have a `Disputed` or `Reported` market status
        /// 2. If status is `Disputed`, then at least one dispute must exist
        InvalidMarketStatus,
    }

    #[pallet::event]
    pub enum Event<T>
    where
        T: Config, {}

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

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
            bond: Self::Balance,
            _: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            _: &Self::MarketId,
            who: &Self::AccountId,
        ) -> DispatchResult {
            CurrencyOf::<T>::reserve(who, bond)?;
            Ok(())
        }

        fn on_resolution<D>(
            bond: &D,
            disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            _: &Self::MarketId,
            market: &Market<Self::AccountId, Self::BlockNumber, MomentOf<T>>,
        ) -> Result<OutcomeReport, DispatchError>
        where
            D: Fn(usize) -> Self::Balance,
        {
            let report = T::MarketCommons::report(market)?;

            // if the market was permissionless and not invalid, return `ValidityBond`.
            // if market.creation == MarketCreation::Permissionless {
            //     if report.outcome != 0 {
            //         CurrencyOf::<T>::unreserve(&market.creator, T::ValidityBond::get());
            //     } else {
            //         // Give it to the treasury instead.
            //         let (imbalance, _) =
            //             CurrencyOf::<T>::slash_reserved(&market.creator, T::ValidityBond::get());
            //         T::Slash::on_unbalanced(imbalance);
            //     }
            // }
            CurrencyOf::<T>::unreserve(&market.creator, T::ValidityBond::get());

            let resolved_outcome = match market.status {
                MarketStatus::Reported => report.outcome.clone(),
                MarketStatus::Disputed => {
                    // count the last dispute's outcome as the winning one
                    if let Some(last_dispute) = disputes.last() {
                        last_dispute.outcome.clone()
                    } else {
                        return Err(Error::<T>::InvalidMarketStatus.into());
                    }
                }
                _ => return Err(Error::<T>::InvalidMarketStatus.into()),
            };

            match market.status {
                MarketStatus::Reported => {
                    // the oracle bond gets returned if the reporter was the oracle
                    if report.by == market.oracle {
                        CurrencyOf::<T>::unreserve(&market.creator, T::OracleBond::get());
                    } else {
                        let (imbalance, _) =
                            CurrencyOf::<T>::slash_reserved(&market.creator, T::OracleBond::get());

                        // give it to the real reporter
                        CurrencyOf::<T>::resolve_creating(&report.by, imbalance);
                    }
                }
                MarketStatus::Disputed => {
                    let mut correct_reporters: Vec<T::AccountId> = Vec::new();

                    let mut overall_imbalance = NegativeImbalanceOf::<T>::zero();

                    // if the reporter reported right, return the OracleBond, otherwise
                    // slash it to pay the correct reporters
                    if report.outcome == resolved_outcome {
                        CurrencyOf::<T>::unreserve(&market.creator, T::OracleBond::get());
                    } else {
                        let (imbalance, _) =
                            CurrencyOf::<T>::slash_reserved(&market.creator, T::OracleBond::get());

                        overall_imbalance.subsume(imbalance);
                    }

                    for (i, dispute) in disputes.iter().enumerate() {
                        let actual_bond = bond(i);
                        if dispute.outcome == resolved_outcome {
                            CurrencyOf::<T>::unreserve(&dispute.by, actual_bond);

                            correct_reporters.push(dispute.by.clone());
                        } else {
                            let (imbalance, _) =
                                CurrencyOf::<T>::slash_reserved(&dispute.by, actual_bond);
                            overall_imbalance.subsume(imbalance);
                        }
                    }

                    // fold all the imbalances into one and reward the correct reporters.
                    let reward_per_each = overall_imbalance
                        .peek()
                        .checked_div(&correct_reporters.len().saturated_into())
                        .ok_or(ArithmeticError::DivisionByZero)?;
                    for correct_reporter in &correct_reporters {
                        let (amount, leftover) = overall_imbalance.split(reward_per_each);
                        CurrencyOf::<T>::resolve_creating(correct_reporter, amount);
                        overall_imbalance = leftover;
                    }
                }
                _ => (),
            };

            Ok(resolved_outcome)
        }
    }

    impl<T> SimpleDisputesPalletApi for Pallet<T> where T: Config {}

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);
}
