#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::NamedReservableCurrency};
    use frame_system::pallet_prelude::*;
    use orml_traits::MultiCurrency;
    use zeitgeist_primitives::{traits::ZeitgeistAssetManager, types::Asset};

    use zrml_market_commons::MarketCommonsPalletApi;

    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;

    pub(crate) type BalanceOf<T> = <<T as Config>::AssetManager as MultiCurrency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Currency: NamedReservableCurrency<Self::AccountId, ReserveIdentifier = [u8; 8]>;

        /// Common market parameters
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;

        /// Shares of outcome assets and native currency
        type AssetManager: ZeitgeistAssetManager<
            Self::AccountId,
            CurrencyId = Asset<MarketIdOf<Self>>,
            ReserveIdentifier = [u8; 8],
        >;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type Burns<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        CurrencyBurned(
            T::AccountId,
            Asset<<<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId>,
            BalanceOf<T>,
        ),
    }

    #[pallet::error]
    pub enum Error<T> {
        FundDoesNotHaveEnoughFreeBalance,
        HasAlreadyBurned,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// WARNING!!: Burns the given amount of ZTG.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn burn(
            origin: OriginFor<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            if Burns::<T>::contains_key(&who) {
                Err(Error::<T>::HasAlreadyBurned)?;
            }

            let free = T::AssetManager::free_balance(Asset::Ztg, &who);

            if free < amount {
                Err(Error::<T>::FundDoesNotHaveEnoughFreeBalance)?;
            }

            T::AssetManager::slash(Asset::Ztg, &who, amount);
            Burns::<T>::insert(&who, true);

            Self::deposit_event(Event::CurrencyBurned(who, Asset::Ztg, amount));

            Ok(())
        }
    }
}
