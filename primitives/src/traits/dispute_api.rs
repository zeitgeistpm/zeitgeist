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

use crate::{market::MarketDispute, outcome_report::OutcomeReport, types::Market};
use frame_support::dispatch::DispatchResult;
use sp_runtime::DispatchError;

pub trait DisputeApi {
    type AccountId;
    type Balance;
    type BlockNumber;
    type MarketId;
    type Moment;
    type Origin;

    /// Initiate a dispute of a reported outcome.
    ///
    /// Further interaction with the dispute API (if necessary) **should** happen through an
    /// associated pallet. **May** assume that `market.dispute_mechanism` refers to the calling dispute API.
    fn on_dispute(
        previous_disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
        market_id: &Self::MarketId,
        market: &Market<Self::AccountId, Self::BlockNumber, Self::Moment>,
    ) -> DispatchResult;

    /// Manage market resolution of a disputed market.
    ///
    /// **Should** only be called if the market was disputed before resolving. **May** assume that
    /// `market.dispute_mechanism` refers to the calling dispute API.
    ///
    /// # Returns
    ///
    /// Returns the dispute mechanism's report if available, otherwise `None`. If `None` is
    /// returned, this means that the dispute could not be resolved.
    fn on_resolution(
        disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
        market_id: &Self::MarketId,
        market: &Market<Self::AccountId, Self::BlockNumber, Self::Moment>,
    ) -> Result<Option<OutcomeReport>, DispatchError>;

    /// Query the future resolution block of a disputed market.
    /// **May** assume that `market.dispute_mechanism` refers to the calling dispute API.
    ///
    /// # Returns
    ///
    /// Returns the future resolution block if available, otherwise `None`.
    fn get_auto_resolve(
        disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
        market_id: &Self::MarketId,
        market: &Market<Self::AccountId, Self::BlockNumber, Self::Moment>,
    ) -> Result<Option<Self::BlockNumber>, DispatchError>;

    /// Query if the dispute mechanism failed for a dispute market.
    /// **May** assume that `market.dispute_mechanism` refers to the calling dispute API.
    ///
    /// # Returns
    ///
    /// Returns `true` if the dispute mechanism failed. Otherwise `false`.
    fn is_fail(
        disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
        market_id: &Self::MarketId,
        market: &Market<Self::AccountId, Self::BlockNumber, Self::Moment>,
    ) -> Result<bool, DispatchError>;
}

pub trait DisputeResolutionApi {
    type AccountId;
    type BlockNumber;
    type MarketId;
    type Moment;

    /// Resolve a market. Fails if `on_resolution` from zrml-prediction-markets fails.
    ///
    /// **Should only be called if the market dispute**
    /// **mechanism is ready for the resolution ([`DisputeApi::on_resolution`]).**
    ///
    /// # Returns
    ///
    /// Returns the consumed weight.
    fn resolve(
        market_id: &Self::MarketId,
        market: &Market<Self::AccountId, Self::BlockNumber, Self::Moment>,
    ) -> Result<u64, DispatchError>;

    /// Add a future block resolution of a disputed market.
    ///
    /// # Returns
    ///
    /// Returns the number of elements in the storage structure.
    fn add_auto_resolve(
        market_id: &Self::MarketId,
        resolution: Self::BlockNumber,
    ) -> Result<u32, DispatchError>;

    /// Remove a future block resolution of a disputed market.
    ///
    /// # Returns
    ///
    /// Returns the number of elements in the storage structure.
    fn remove_auto_resolve(market_id: &Self::MarketId, resolution: Self::BlockNumber) -> u32;
}
