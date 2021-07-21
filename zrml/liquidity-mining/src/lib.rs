//! Manages and distributes incentives to liquidity providers
//!
//! Each block has a maximum allowed amount of ZTG that is distributed among the`PoolShare`
//! owners of that same block. Over time this amount will increase until a market closes and
//! then all rewards will be distributed accordingly.
//!
//! This pallet is mostly self-contained and only need to know about the native currency. To
//! interact with its functionalities, please use the provided API.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod benchmarks;
mod liquidity_mining_pallet_api;
mod mock;
mod owned_values_params;
mod tests;
mod track_incentives_based_on_bought_shares;
mod track_incentives_based_on_sold_shares;
mod utils;
pub mod weights;

pub use liquidity_mining_pallet_api::LiquidityMiningPalletApi;
pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::{
        owned_values_params::OwnedValuesParams,
        track_incentives_based_on_bought_shares::TrackIncentivesBasedOnBoughtShares,
        track_incentives_based_on_sold_shares::TrackIncentivesBasedOnSoldShares,
        utils::{calculate_perthousand, calculate_perthousand_value, perthousand_to_balance},
        weights::WeightInfoZeitgeist,
        LiquidityMiningPalletApi,
    };
    use alloc::vec::Vec;
    use core::marker::PhantomData;
    #[cfg(feature = "std")]
    use frame_support::traits::GenesisBuild;
    use frame_support::{
        dispatch::DispatchResult,
        log,
        storage::{
            types::{StorageDoubleMap, StorageMap, StorageValue, ValueQuery},
            with_transaction,
        },
        traits::{
            Currency, ExistenceRequirement, Get, Hooks, IsType, ReservableCurrency, WithdrawReasons,
        },
        Blake2_128Concat, PalletId, Twox64Concat,
    };
    use frame_system::{ensure_root, pallet_prelude::OriginFor};
    use sp_runtime::{
        traits::{AccountIdConversion, Saturating, Zero},
        TransactionOutcome,
    };
    use zeitgeist_primitives::traits::MarketId;

    pub(crate) type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(T::WeightInfo::set_per_block_distribution())]
        pub fn set_per_block_distribution(
            origin: OriginFor<T>,
            per_block_distribution: BalanceOf<T>,
        ) -> DispatchResult {
            let _ = ensure_root(origin)?;
            <PerBlockIncentive<T>>::put(per_block_distribution);
            Ok(())
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Currency: ReservableCurrency<Self::AccountId>;
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type MarketId: MarketId;
        type PalletId: Get<PalletId>;
        type WeightInfo: WeightInfoZeitgeist;
    }

    #[pallet::event]
    pub enum Event<T>
    where
        T: Config, {}

    #[pallet::error]
    pub enum Error<T> {}

    #[cfg(feature = "std")]
    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            T::Currency::deposit_creating(&Pallet::<T>::pallet_account_id(), self.initial_balance);
            <PerBlockIncentive<T>>::put(self.per_block_distribution);
        }
    }

    #[cfg(feature = "std")]
    #[derive(Debug)]
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub initial_balance: BalanceOf<T>,
        pub per_block_distribution: BalanceOf<T>,
    }

    #[cfg(feature = "std")]
    impl<T> Default for GenesisConfig<T>
    where
        T: Config,
    {
        #[inline]
        fn default() -> Self {
            Self {
                initial_balance: BalanceOf::<T>::zero(),
                per_block_distribution: BalanceOf::<T>::zero(),
            }
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        /// Manages incentives on each block finalization.
        fn on_finalize(block: T::BlockNumber) {
            let fun = || {
                TrackIncentivesBasedOnBoughtShares::<T>::exec(block)?;
                TrackIncentivesBasedOnSoldShares::<T>::exec();
                Some(())
            };
            with_transaction(|| match fun() {
                None => {
                    log::error!("Block {:?} was not finalized", block);
                    TransactionOutcome::Rollback(())
                }
                Some(_) => TransactionOutcome::Commit(()),
            });
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    impl<T> Pallet<T>
    where
        T: Config,
    {
        // pot/fund account
        pub(crate) fn pallet_account_id() -> T::AccountId {
            T::PalletId::get().into_account()
        }
    }

    impl<T> LiquidityMiningPalletApi for Pallet<T>
    where
        T: Config,
    {
        type AccountId = T::AccountId;
        type Balance = BalanceOf<T>;
        type BlockNumber = T::BlockNumber;
        type MarketId = T::MarketId;

        fn add_market_period(market_id: Self::MarketId, period: [Self::BlockNumber; 2]) {
            <Markets<T>>::insert(market_id, period)
        }

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
                            let [start, end] = <Markets<T>>::get(market_id)?;
                            let market_blocks = end.saturating_sub(start);
                            let ptd = calculate_perthousand(participated_blocks, &market_blocks)?;
                            let ptd_balance = perthousand_to_balance(ptd);
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

            let fund_does_not_have_enough_balance = T::Currency::ensure_can_withdraw(
                &pallet_account_id,
                final_total_incentives,
                WithdrawReasons::all(),
                T::Currency::free_balance(&pallet_account_id)
                    .saturating_sub(final_total_incentives),
            )
            .is_err();
            if fund_does_not_have_enough_balance {
                return Ok(());
            }

            for (account_id, incentives) in values {
                T::Currency::transfer(
                    &pallet_account_id,
                    &account_id,
                    incentives,
                    ExistenceRequirement::AllowDeath,
                )?;
            }
            <Markets<T>>::remove(&market_id);
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

    /// Shares bought in the current block being constructed. Automatically erased after each finalized block.
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

    /// Shares sold in the current block being constructed. Automatically erased after each finalized block.
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

    /// Market end periods
    #[pallet::storage]
    pub type Markets<T: Config> = StorageMap<_, Blake2_128Concat, T::MarketId, [T::BlockNumber; 2]>;

    /// Owned balances (not shares) that are going to be distributed as incentives. Automatically
    /// updated after each finalized block.
    #[pallet::storage]
    pub type OwnedValues<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::MarketId,
        Twox64Concat,
        T::AccountId,
        OwnedValuesParams<BalanceOf<T>, T::BlockNumber>,
        ValueQuery,
    >;

    /// Per block distribution. How much each block will distribute across bought shares.
    #[pallet::storage]
    pub type PerBlockIncentive<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;
}
