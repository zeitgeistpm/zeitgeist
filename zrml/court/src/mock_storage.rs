// Copyright 2023 Forecasting Technologies LTD.
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

#![cfg(test)]
#![allow(dead_code)]

pub use pallet::*;
use parity_scale_codec::Encode;
use sp_runtime::traits::Hash;

#[frame_support::pallet]
pub(crate) mod pallet {
    use core::marker::PhantomData;
    use frame_support::pallet_prelude::*;
    use zrml_market_commons::MarketCommonsPalletApi;

    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;

    pub type CacheSize = ConstU32<64>;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    /// Only used for testing the dispute resolution API to prediction-markets
    #[pallet::storage]
    pub(crate) type MarketIdsPerDisputeBlock<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::BlockNumber,
        BoundedVec<MarketIdOf<T>, CacheSize>,
        ValueQuery,
    >;
}

impl<T: Config> frame_support::traits::Randomness<T::Hash, T::BlockNumber> for Pallet<T> {
    fn random(subject: &[u8]) -> (T::Hash, T::BlockNumber) {
        let block_number = <frame_system::Pallet<T>>::block_number();
        let seed = subject.using_encoded(T::Hashing::hash);

        (seed, block_number)
    }
}
