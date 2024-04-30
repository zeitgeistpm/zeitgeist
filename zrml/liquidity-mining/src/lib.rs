// Copyright 2022-2024 Forecasting Technologies LTD.
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

#[macro_use]
mod utils;

mod benchmarks;
mod liquidity_mining_pallet_api;
mod mock;
mod owned_values_params;
mod tests;
mod track_incentives_based_on_bought_shares;
mod track_incentives_based_on_sold_shares;
pub mod weights;

pub use liquidity_mining_pallet_api::LiquidityMiningPalletApi;
pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::{
        owned_values_params::OwnedValuesParams,
        track_incentives_based_on_bought_shares::TrackIncentivesBasedOnBoughtShares,
        track_incentives_based_on_sold_shares::TrackIncentivesBasedOnSoldShares,
        utils::{
            calculate_average_blocks_of_a_time_period, calculate_perthousand,
            calculate_perthousand_value,
        },
        weights::WeightInfoZeitgeist,
        LiquidityMiningPalletApi,
    };
    use alloc::vec::Vec;
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::DispatchResult,
        storage::{
            types::{StorageDoubleMap, StorageValue, ValueQuery},
            with_transaction,
        },
        traits::{
            BuildGenesisConfig, Currency, ExistenceRequirement, Get, Hooks, IsType, StorageVersion,
            WithdrawReasons,
        },
        Blake2_128Concat, PalletId, Twox64Concat,
    };
    use frame_system::{
        ensure_root,
        pallet_prelude::{BlockNumberFor, OriginFor},
    };
    use sp_runtime::{
        traits::{AccountIdConversion, Saturating},
        TransactionOutcome,
    };
    use zeitgeist_primitives::{
        traits::MarketId,
        types::{MarketPeriod, MaxRuntimeUsize},
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);
    const LOG_TARGET: &str = "runtime::zrml-liquidity-mining";

    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
    pub(crate) type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::set_per_block_distribution())]
        // MARK(non-transactional): `set_per_block_distribution` is infallible.
        pub fn set_per_block_distribution(
            origin: OriginFor<T>,
            #[pallet::compact] per_block_distribution: BalanceOf<T>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            <PerBlockIncentive<T>>::put(per_block_distribution);
            Ok(())
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Currency: Currency<Self::AccountId>;

        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type MarketCommons: MarketCommonsPalletApi<
                AccountId = Self::AccountId,
                BlockNumber = BlockNumberFor<Self>,
                MarketId = Self::MarketId,
                Balance = BalanceOf<Self>,
            >;

        type MarketId: MarketId;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        type WeightInfo: WeightInfoZeitgeist;
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// The number of markets that received incentives in a block
        AddedIncentives(MaxRuntimeUsize),
        /// The total amount of incentives distributed to accounts along side the number
        /// of accounts that received these incentives.
        DistributedIncentives(BalanceOf<T>, MaxRuntimeUsize),
        /// The number of markets that subtracted incentives in a block
        SubtractedIncentives(MaxRuntimeUsize),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Pallet account does not have enough funds
        FundDoesNotHaveEnoughBalance,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            let _ = T::Currency::deposit_creating(
                &Pallet::<T>::pallet_account_id(),
                self.initial_balance,
            );
            <PerBlockIncentive<T>>::put(self.per_block_distribution);
        }
    }

    #[derive(scale_info::TypeInfo, Debug)]
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub initial_balance: BalanceOf<T>,
        pub per_block_distribution: BalanceOf<T>,
    }

    impl<T> Default for GenesisConfig<T>
    where
        T: Config,
    {
        #[inline]
        fn default() -> Self {
            Self {
                initial_balance: BalanceOf::<T>::from(0u8),
                per_block_distribution: BalanceOf::<T>::from(0u8),
            }
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // Manages incentives on each block finalization.
        fn on_finalize(block: BlockNumberFor<T>) {
            let fun = || {
                let added_len = TrackIncentivesBasedOnBoughtShares::<T>::exec(block)?;
                if added_len > 0 {
                    Self::deposit_event(Event::AddedIncentives(added_len.into()));
                }
                let subtracted_len = TrackIncentivesBasedOnSoldShares::<T>::exec();
                if subtracted_len > 0 {
                    Self::deposit_event(Event::SubtractedIncentives(subtracted_len.into()));
                }
                Some(())
            };
            let _ = with_transaction(|| match fun() {
                None => {
                    log::error!(target: LOG_TARGET, "Block {:?} was not finalized", block);
                    TransactionOutcome::Rollback(Err("Block was not finalized"))
                }
                Some(_) => TransactionOutcome::Commit(Ok(())),
            });
        }
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    impl<T> Pallet<T>
    where
        T: Config,
    {
        // pot/fund account
        #[inline]
        pub(crate) fn pallet_account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }
    }

    impl<T> LiquidityMiningPalletApi for Pallet<T>
    where
        T: Config,
    {
        type AccountId = T::AccountId;
        type Balance = BalanceOf<T>;
        type BlockNumber = BlockNumberFor<T>;
        type MarketId = T::MarketId;

        fn add_shares(
            account_id: Self::AccountId,
            market_id: Self::MarketId,
            shares: Self::Balance,
        ) {
            <BlockBoughtShares<T>>::mutate(market_id, account_id, |total_shares| {
                *total_shares = total_shares.saturating_add(shares);
            })
        }

        fn distribute_market_incentives(market_id: &Self::MarketId) -> DispatchResult {
            let pallet_account_id = Pallet::<T>::pallet_account_id();
            let mut final_total_incentives = BalanceOf::<T>::from(0u8);

            let values: Vec<_> = <OwnedValues<T>>::drain_prefix(market_id)
                .filter_map(
                    |(
                        account_id,
                        OwnedValuesParams {
                            participated_blocks,
                            perpetual_incentives,
                            total_incentives: local_total_incentives,
                            ..
                        },
                    )| {
                        let actual_perpetual_incentives = {
                            let opt = match T::MarketCommons::market(market_id).ok()?.period {
                                MarketPeriod::Block(range) => calculate_perthousand(
                                    participated_blocks,
                                    &range.end.saturating_sub(range.start),
                                ),
                                MarketPeriod::Timestamp(range) => calculate_perthousand(
                                    participated_blocks,
                                    &calculate_average_blocks_of_a_time_period::<T>(&range),
                                ),
                            };
                            let ptd_balance = opt.map(|ptd| ptd.into())?;
                            calculate_perthousand_value(ptd_balance, perpetual_incentives)
                        };
                        let final_incentives =
                            actual_perpetual_incentives.saturating_add(local_total_incentives);
                        final_total_incentives =
                            final_total_incentives.saturating_add(final_incentives);
                        Some((account_id, final_incentives))
                    },
                )
                .collect();

            T::Currency::ensure_can_withdraw(
                &pallet_account_id,
                final_total_incentives,
                WithdrawReasons::all(),
                T::Currency::free_balance(&pallet_account_id)
                    .saturating_sub(final_total_incentives),
            )
            .map_err(|_err| Error::<T>::FundDoesNotHaveEnoughBalance)?;

            let accounts_len = values.len().into();
            for (account_id, incentives) in values {
                T::Currency::transfer(
                    &pallet_account_id,
                    &account_id,
                    incentives,
                    ExistenceRequirement::AllowDeath,
                )
                .map_err(|_err| Error::<T>::FundDoesNotHaveEnoughBalance)?;
            }
            Self::deposit_event(Event::DistributedIncentives(final_total_incentives, accounts_len));
            Ok(())
        }

        fn remove_shares(
            account_id: &Self::AccountId,
            market_id: &Self::MarketId,
            shares: Self::Balance,
        ) {
            <BlockSoldShares<T>>::mutate(market_id, account_id, |total_shares| {
                *total_shares = total_shares.saturating_add(shares);
            })
        }
    }

    /// Shares bought in the current block being constructed. Automatically *erased* after each finalized block.
    #[pallet::storage]
    pub type BlockBoughtShares<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::MarketId,
        Twox64Concat,
        T::AccountId,
        BalanceOf<T>,
        ValueQuery,
    >;

    /// Shares sold in the current block being constructed. Automatically *erased* after each finalized block.
    #[pallet::storage]
    pub type BlockSoldShares<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::MarketId,
        Twox64Concat,
        T::AccountId,
        BalanceOf<T>,
        ValueQuery,
    >;

    /// Owned balances (not shares) that are going to be distributed as incentives. Automatically
    /// *updated* after each finalized block.
    #[pallet::storage]
    pub type OwnedValues<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::MarketId,
        Twox64Concat,
        T::AccountId,
        OwnedValuesParams<BalanceOf<T>, BlockNumberFor<T>>,
        ValueQuery,
    >;

    /// Per block distribution. How much each block will distribute across bought shares.
    #[pallet::storage]
    pub type PerBlockIncentive<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;
}
