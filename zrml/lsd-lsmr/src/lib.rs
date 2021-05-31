//! # Court
//!
//! Manages market disputes and resolutions.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod traits;
pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use core::marker::PhantomData;
    use frame_support::{
        pallet_prelude::{MaybeSerializeDeserialize, Member},
        traits::{Get, Hooks, IsType, Time},
        Parameter,
    };
    use parity_scale_codec::Codec;
    use sp_runtime::traits::{AtLeast32Bit, AtLeast32BitUnsigned, Hash};
    use sp_std::fmt::Debug;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The type that offers timestamping functionality
        type Timestamp: Time;

        /// The type of timestamps
        type Moment: AtLeast32Bit + Parameter + Default + Copy;

        /// Short EMA time range (in seconds)
        type EmaShort: Get<Self::Moment>;

        /// Long Ema time range (in seconds)
        type EmaLong: Get<Self::Moment>;

        /// Ema smoothing factor
        type Smoothing: Get<Self::Moment>;

        /// Asset type
        type Asset: Eq + Hash + PartialEq;

        /// Balance type: Defines the type of traded amounts
        type Balance: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Codec
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + Debug;
    }

    #[pallet::error]
    pub enum Error<T> {}

    #[pallet::event]
    //#[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config, {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    impl<T: Config> Pallet<T> {}

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);
}
