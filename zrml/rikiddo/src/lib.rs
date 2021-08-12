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
    use core::{fmt::Debug, marker::PhantomData};
    use frame_support::{
        pallet_prelude::{MaybeSerializeDeserialize, Member},
        traits::{Hooks, Time},
        Parameter,
    };
    use parity_scale_codec::Codec;
    use sp_runtime::traits::AtLeast32BitUnsigned;
    use substrate_fixed::types::extra::LeEqU128;

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

        /// The type that will be used for the fractional part of the fixed point numbers
        type FractionalType: LeEqU128;
    }

    #[pallet::error]
    pub enum Error<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    impl<T: Config> Pallet<T> {}

    // This is the storage containing the Rikiddo instances per pool.
    /*
    #[pallet::storage]
    pub type LmsrPerPool<T: Config> = StorageMap<
        _,
        Twox64Concat,
        u128,
        RikiddoMV<
            FixedU128<T::FractionalType>,
            FeeSigmoid,
            EmaMarketVolume<FixedU128<T::FractionalType>>,
        >,
    >;
    */

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);
}
