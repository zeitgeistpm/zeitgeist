#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

extern crate alloc;

mod mock;
mod tests;

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use orml_traits::MultiCurrency;
    use sp_runtime::SaturatedConversion;
    use zeitgeist_primitives::{traits::ZeitgeistAssetManager, types::Asset};
    use zrml_market_commons::MarketCommonsPalletApi;

    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;

    pub(crate) type BalanceOf<T> = <<T as Config>::AssetManager as MultiCurrency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The origin that is allowed to destroy markets.
        type SetBurnAmountOrigin: EnsureOrigin<Self::Origin>;

        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

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

    /// Keep track of crossings. Accounts are only able to cross once.
    #[pallet::storage]
    pub type Crossings<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool>;

    #[pallet::type_value]
    pub fn DefaultBurnAmount<T: Config>() -> BalanceOf<T> {
        (zeitgeist_primitives::constants::BASE * 100).saturated_into()
    }

    /// An extra layer of pseudo randomness.
    #[pallet::storage]
    pub type BurnAmount<T: Config> =
        StorageValue<_, BalanceOf<T>, ValueQuery, DefaultBurnAmount<T>>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A account crossed and claimed their right to create their avatar.
        AccountCrossed(
            T::AccountId,
            Asset<<<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId>,
            BalanceOf<T>,
        ),
        /// The crossing fee was changed.
        CrossingFeeChanged(
            T::AccountId,
            Asset<<<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId>,
            BalanceOf<T>,
        ),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Account does not have enough balance to cross.
        FundDoesNotHaveEnoughFreeBalance,
        /// Account has already crossed.
        HasAlreadyCrossed,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Burns 200 ZTG to cross, granting the ability to claim your zeitgeist avatar.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn cross(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            if Crossings::<T>::contains_key(&who) {
                Err(Error::<T>::HasAlreadyCrossed)?;
            }

            let amount: BalanceOf<T> = BurnAmount::<T>::get().saturated_into();
            let free = T::AssetManager::free_balance(Asset::Ztg, &who);

            if free < amount {
                Err(Error::<T>::FundDoesNotHaveEnoughFreeBalance)?;
            }

            T::AssetManager::slash(Asset::Ztg, &who, amount);
            Crossings::<T>::insert(&who, true);

            Self::deposit_event(Event::AccountCrossed(who, Asset::Ztg, amount));

            Ok(())
        }

        /// Set the burn amount. Needs 50% council vote.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_burn_amount(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin.clone())?;
            T::SetBurnAmountOrigin::ensure_origin(origin)?;
            BurnAmount::<T>::put(amount);
            Self::deposit_event(Event::CrossingFeeChanged(who, Asset::Ztg, amount));
            Ok(())
        }
    }
}
