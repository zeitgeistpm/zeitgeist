//! # Court
//!
//! Manages market disputes and resolutions.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod constants;
mod mock;
mod tests;
pub mod traits;
pub mod types;
pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use core::marker::PhantomData;
    use frame_support::{
        pallet_prelude::{MaybeSerializeDeserialize, Member},
        traits::{Hooks, Time},
        Parameter,
    };
    use parity_scale_codec::Codec;
    use sp_runtime::traits::AtLeast32BitUnsigned;
    use sp_std::fmt::Debug;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Balance type: Defines the type of traded amounts
        type Balance: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Codec
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + Debug;

        /// The type that offers timestamping functionality
        type Timestamp: Time;
    }

    #[pallet::error]
    pub enum Error<T> {}

    /*#[pallet::event]
    //#[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config, {}*/

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    impl<T: Config> Pallet<T> {}

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);
}
