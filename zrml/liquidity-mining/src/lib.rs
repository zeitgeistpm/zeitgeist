//! Your mom

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(test)]
mod mock;
mod shares_params;
#[cfg(test)]
mod tests;

pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::shares_params::SharesParams;
    use alloc::vec::Vec;
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::{DispatchError, DispatchResult, DispatchResultWithPostInfo},
        storage::types::{StorageDoubleMap, StorageValue, ValueQuery},
        traits::{
            Currency, ExistenceRequirement, GenesisBuild, Get, Hooks, IsType, ReservableCurrency,
        },
        Blake2_128Concat, PalletId, Twox64Concat,
    };
    use frame_system::{ensure_root, pallet_prelude::OriginFor};
    use sp_runtime::traits::{AccountIdConversion, CheckedAdd, CheckedDiv, CheckedMul, Zero};

    pub(crate) type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn set_per_block_distribution(
            origin: OriginFor<T>,
            per_block_distribution: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            <PerBlockDistribution<T>>::put(per_block_distribution);
            Ok(Some(0).into())
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Currency: ReservableCurrency<Self::AccountId>;
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type PalletId: Get<PalletId>;
    }

    #[pallet::error]
    pub enum Error<T> {}

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        BadBalanceTransfer(DispatchError),
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            T::Currency::deposit_creating(&Pallet::<T>::pallet_account_id(), self.initial_balance);
            <PerBlockDistribution<T>>::put(self.per_block_distribution);
        }
    }

    #[derive(Debug)]
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
                initial_balance: BalanceOf::<T>::zero(),
                per_block_distribution: BalanceOf::<T>::zero(),
            }
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_finalize(_: T::BlockNumber) {
            let per_block_distribution = <PerBlockDistribution<T>>::get();

            let mut bought_shares = <BlockBoughtShares<T>>::iter().collect::<Vec<_>>();
            let mut total_bought_shares = BalanceOf::<T>::zero();
            let mut initial_invalid_idx = None;

            bought_shares.sort_unstable_by(|(.., a), (.., b)| a.cmp(&b));

            for (idx, (.., bought_shares)) in bought_shares.iter().enumerate() {
                let opt = total_bought_shares.checked_add(&bought_shares);
                if let Some(el) = opt {
                    total_bought_shares = el;
                } else {
                    initial_invalid_idx = Some(idx);
                    break;
                }
            }

            let range = initial_invalid_idx.unwrap_or(bought_shares.len())..;
            let mut remaining_bought_shares = bought_shares.drain(range).collect::<Vec<_>>();

            let opt = Self::buy_share_value(&per_block_distribution, &total_bought_shares);
            let buy_share_value = if let Some(el) = opt {
                el
            } else {
                return;
            };

            for (account_id, pool_id, bought_shares) in bought_shares {
                let amount = if let Some(el) = buy_share_value.checked_mul(&bought_shares) {
                    el
                } else {
                    // Someone is buying too many shares, way beyond what `Balance` can hold
                    remaining_bought_shares.push((account_id, pool_id, bought_shares));
                    continue;
                };

                let rslt = <OwnedBalances<T>>::try_mutate(pool_id, account_id, |owned_balance| {
                    *owned_balance =
                        owned_balance
                            .checked_add(&amount)
                            .ok_or(DispatchError::Other(
                                "Adding more balance would cause an overflow",
                            ))?;
                    Ok(())
                });

                if let Err(err) = rslt {
                    Self::deposit_event(Event::BadBalanceTransfer(err));
                }
            }

            <BlockBoughtShares<T>>::remove_all();
            for (account, pool_id, bought_shares) in remaining_bought_shares {
                <BlockBoughtShares<T>>::insert(account, pool_id, bought_shares);
            }
            <BlockSoldShares<T>>::remove_all();
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    impl<T> Pallet<T>
    where
        T: Config,
    {
        #[frame_support::transactional]
        pub fn remove_pool_distributing_rewards(pool_id: u128) -> DispatchResult {
            let pallet_account_id = Pallet::<T>::pallet_account_id();
            for (account_id, owned_balances) in <OwnedBalances<T>>::drain_prefix(pool_id) {
                T::Currency::transfer(
                    &pallet_account_id,
                    &account_id,
                    owned_balances,
                    ExistenceRequirement::KeepAlive,
                )?;
            }
            Ok(())
        }

        pub(crate) fn pallet_account_id() -> T::AccountId {
            T::PalletId::get().into_account()
        }

        // ZTG value of one bought share for the current block being produced. Or in other others:
        // Determines how much a share will be worth given the amount of ZTG for liquidity
        // mining and the total number of bought shares for the current block.
        //
        // `None` result means no-one purchased a share.
        fn buy_share_value(
            per_block_distribution: &BalanceOf<T>,
            total_bought_shares: &BalanceOf<T>,
        ) -> Option<BalanceOf<T>> {
            per_block_distribution.checked_div(&total_bought_shares)
        }
    }

    /// Shares bought in the current block being constructed. Automatically erased after each finished block.
    #[pallet::storage]
    #[pallet::getter(fn block_bought_shares)]
    pub type BlockBoughtShares<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Twox64Concat,
        u128,
        BalanceOf<T>,
        ValueQuery,
    >;

    /// Shares sold in the current block being constructed. Automatically erased after each finished block.
    #[pallet::storage]
    #[pallet::getter(fn block_sold_shares)]
    pub type BlockSoldShares<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Twox64Concat,
        u128,
        SharesParams<BalanceOf<T>>,
        ValueQuery,
    >;

    /// Owned balances (not shares) that are going to be distributed as rewards. Automatically
    /// updated after each finished block.
    #[pallet::storage]
    #[pallet::getter(fn owned_shares)]
    pub type OwnedBalances<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u128,
        Twox64Concat,
        T::AccountId,
        BalanceOf<T>,
        ValueQuery,
    >;

    /// Per block distribution. How much rewards each block will distribute.
    #[pallet::storage]
    #[pallet::getter(fn per_block_distribution)]
    pub type PerBlockDistribution<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;
}
