//! # Court

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod mock;

pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use core::marker::PhantomData;
    use frame_support::traits::{Hooks, IsType};

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::error]
    pub enum Error<T> {}

    #[pallet::event]
    pub enum Event<T>
    where
        T: Config, {}

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);
}
