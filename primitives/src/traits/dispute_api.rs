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

use crate::{
    market::MarketDispute,
    outcome_report::OutcomeReport,
    types::{Asset, Market},
};
use frame_support::dispatch::DispatchResult;
use parity_scale_codec::MaxEncodedLen;
use sp_runtime::DispatchError;

pub trait DisputeApi {
    type AccountId;
    type Balance;
    type BlockNumber;
    type MarketId: MaxEncodedLen;
    type Moment;
    type Origin;

    /// Initiate a dispute of a reported outcome.
    ///
    /// Further interaction with the dispute API (if necessary) **should** happen through an
    /// associated pallet. **May** assume that `market.dispute_mechanism` refers to the calling dispute API.
    fn on_dispute(
        previous_disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
        market_id: &Self::MarketId,
        market: &Market<Self::AccountId, Self::BlockNumber, Self::Moment, Asset<Self::MarketId>>,
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
        market: &Market<Self::AccountId, Self::BlockNumber, Self::Moment, Asset<Self::MarketId>>,
    ) -> Result<Option<OutcomeReport>, DispatchError>;
}
