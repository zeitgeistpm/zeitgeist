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
    use core::marker::PhantomData;
    use frame_support::{
        ensure,
        traits::{Get, IsType, StorageVersion},
        PalletId,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use orml_traits::MultiCurrency;
    use sp_runtime::{traits::AccountIdConversion, DispatchResult};
    use zeitgeist_primitives::{
        traits::DistributeFees,
        types::{Asset, Market, MarketStatus, MarketType, Outcome, ScalarPosition, ScoringRule},
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type BalanceOf<T> =
        <<T as Config>::AssetManager as MultiCurrency<AccountIdOf<T>>>::Balance;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub(crate) type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;
    pub(crate) type MarketOf<T> = Market<
        AccountIdOf<T>,
        BalanceOf<T>,
        <T as frame_system::Config>::BlockNumber,
        MomentOf<T>,
        Asset<MarketIdOf<T>>,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
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
            ensure!(market.status == MarketStatus::Active, Error::<T>::MarketIsNotActive);
            ensure!(market.scoring_rule == ScoringRule::Parimutuel, Error::<T>::InvalidScoringRule);

            let market_assets = Self::outcome_assets(market_id, &market);
            ensure!(market_assets.binary_search(&asset).is_ok(), Error::<T>::InvalidOutcomeAsset);

            let pot_account = Self::pot_account(market_id);
            T::AssetManager::transfer(market.base_asset, &who, &pot_account, amount)?;

            T::AssetManager::deposit(asset, &who, amount)?;

            Self::deposit_event(Event::OutcomeBought { market_id, asset });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(5000)]
        #[frame_support::transactional]
        pub fn claim_rewards(origin: OriginFor<T>, market_id: MarketIdOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let market = T::MarketCommons::market(&market_id)?;

            Self::deposit_event(Event::RewardsClaimed { market_id });

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

        type AssetManager: MultiCurrency<Self::AccountId, CurrencyId = Asset<MarketIdOf<Self>>>;

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
        OutcomeMismatch,
        MarketIsNotActive,
        AmountTooSmall,
        NotParimutuelOutcome,
        InvalidOutcomeAsset,
        InvalidScoringRule,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        OutcomeBought { market_id: MarketIdOf<T>, asset: Asset<MarketIdOf<T>> },
        RewardsClaimed { market_id: MarketIdOf<T> },
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

        pub fn outcome_assets(
            market_id: MarketIdOf<T>,
            market: &MarketOf<T>,
        ) -> Vec<Asset<MarketIdOf<T>>> {
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
        scoring_rule: ScoringRule::CPMM,
        status: MarketStatus::Disputed,
        bonds: MarketBonds::default(),
    }
}
