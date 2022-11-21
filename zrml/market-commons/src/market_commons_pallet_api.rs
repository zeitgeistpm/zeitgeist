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

#![allow(clippy::type_complexity)]
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    pallet_prelude::{MaybeSerializeDeserialize, Member},
    storage::PrefixIterator,
    traits::{Currency, NamedReservableCurrency},
    Parameter,
};
use parity_scale_codec::MaxEncodedLen;
use sp_runtime::traits::AtLeast32Bit;
use zeitgeist_primitives::types::{Market, PoolId};

// Abstraction of the market type, which is not a part of `MarketCommonsPalletApi` because Rust
// doesn't support type aliases in traits.
type MarketOf<T> = Market<
    <T as MarketCommonsPalletApi>::AccountId,
    <<T as MarketCommonsPalletApi>::Currency as Currency<
        <T as MarketCommonsPalletApi>::AccountId,
    >>::Balance,
    <T as MarketCommonsPalletApi>::BlockNumber,
    <T as MarketCommonsPalletApi>::Moment,
>;

pub trait MarketCommonsPalletApi {
    type AccountId;
    type BlockNumber: AtLeast32Bit;
    type Currency: NamedReservableCurrency<Self::AccountId, ReserveIdentifier = [u8; 8]>;
    type MarketId: AtLeast32Bit
        + Copy
        + Default
        + MaybeSerializeDeserialize
        + MaxEncodedLen
        + Member
        + Parameter;
    type Moment: AtLeast32Bit + Copy + Default + Parameter + MaxEncodedLen;

    // Market

    /// Latest attributed auto-generated ID
    ///
    /// Returns `Err` if no market has bees created
    fn latest_market_id() -> Result<Self::MarketId, DispatchError>;

    /// Return an iterator over the key-value pairs of markets. Altering market storage during
    /// iteration results in undefined behavior.
    fn market_iter() -> PrefixIterator<(Self::MarketId, MarketOf<Self>)>;

    /// Gets a market from the storage.
    fn market(market_id: &Self::MarketId) -> Result<MarketOf<Self>, DispatchError>;

    /// Mutates a given market storage
    fn mutate_market<F>(market_id: &Self::MarketId, cb: F) -> DispatchResult
    where
        F: FnOnce(&mut MarketOf<Self>) -> DispatchResult;

    /// Pushes a new market into the storage, returning its related auto-incremented ID.
    fn push_market(market: MarketOf<Self>) -> Result<Self::MarketId, DispatchError>;

    /// Removes a market from the storage.
    fn remove_market(market_id: &Self::MarketId) -> DispatchResult;

    /// Return the account id of a market's prize pool.
    fn market_account(market_id: Self::MarketId) -> Self::AccountId;

    // MarketPool

    /// Connects a pool identified by `pool_id` to a market identified by `market_id`.
    fn insert_market_pool(market_id: Self::MarketId, pool_id: PoolId) -> DispatchResult;

    /// Removes the pool id associated with a given `market_id`
    fn remove_market_pool(market_id: &Self::MarketId) -> DispatchResult;

    /// Fetches the pool id associated with a given `market_id`.
    fn market_pool(market_id: &Self::MarketId) -> Result<PoolId, DispatchError>;

    // Etc

    /// Returns the current UTC time (milliseconds)
    fn now() -> Self::Moment;
}
