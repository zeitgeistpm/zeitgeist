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

use crate::{outcome_report::OutcomeReport, types::Market};
use frame_support::{dispatch::DispatchResult, pallet_prelude::Weight};
use sp_runtime::DispatchError;

// Abstraction of the market type, which is not a part of `DisputeApi` because Rust doesn't support
// type aliases in traits.
type MarketOfDisputeApi<T> = Market<
    <T as DisputeApi>::AccountId,
    <T as DisputeApi>::Balance,
    <T as DisputeApi>::BlockNumber,
    <T as DisputeApi>::Moment,
>;

pub trait DisputeApi {
    type AccountId;
    type Balance;
    type NegativeImbalance;
    type BlockNumber;
    type MarketId;
    type Moment;
    type Origin;

    /// Initiate a dispute of a reported outcome.
    ///
    /// Further interaction with the dispute API (if necessary) **should** happen through an
    /// associated pallet. **May** assume that `market.dispute_mechanism` refers to the calling dispute API.
    fn on_dispute(market_id: &Self::MarketId, market: &MarketOfDisputeApi<Self>) -> DispatchResult;

    /// Manage market resolution of a disputed market.
    ///
    /// **Should** only be called if the market was disputed before resolving. **May** assume that
    /// `market.dispute_mechanism` refers to the calling dispute API.
    ///
    /// # Returns
    ///
    /// Returns the dispute mechanism's report if available, otherwise `None`. If `None` is
    /// returned, this means that the dispute could not be resolved.
    fn get_resolution_outcome(
        market_id: &Self::MarketId,
        market: &MarketOfDisputeApi<Self>,
    ) -> Result<Option<OutcomeReport>, DispatchError>;

    /// Allow the flow of funds to the market dispute mechanism.
    /// **May** assume that `market.dispute_mechanism` refers to the calling dispute API.
    ///
    /// # Returns
    /// Returns the negative imbalance which is meant to be used for the treasury.
    fn maybe_pay(
        market_id: &Self::MarketId,
        market: &MarketOfDisputeApi<Self>,
        resolved_outcome: &OutcomeReport,
        overall_imbalance: Self::NegativeImbalance,
    ) -> Result<Self::NegativeImbalance, DispatchError>;

    /// Query the future resolution block of a disputed market.
    /// **May** assume that `market.dispute_mechanism` refers to the calling dispute API.
    ///
    /// # Returns
    ///
    /// Returns the future resolution block if available, otherwise `None`.
    fn get_auto_resolve(
        market_id: &Self::MarketId,
        market: &MarketOfDisputeApi<Self>,
    ) -> Result<Option<Self::BlockNumber>, DispatchError>;

    /// Returns `true` if the market dispute mechanism
    /// was unable to come to a conclusion.
    /// **May** assume that `market.dispute_mechanism` refers to the calling dispute API.
    fn has_failed(
        market_id: &Self::MarketId,
        market: &MarketOfDisputeApi<Self>,
    ) -> Result<bool, DispatchError>;

    /// Called, when a global dispute is started.
    /// **May** assume that `market.dispute_mechanism` refers to the calling dispute API.
    fn on_global_dispute(
        market_id: &Self::MarketId,
        market: &MarketOfDisputeApi<Self>,
    ) -> DispatchResult;

    /// Called, when a market is destroyed.
    /// **May** assume that `market.dispute_mechanism` refers to the calling dispute API.
    fn clear(market_id: &Self::MarketId, market: &MarketOfDisputeApi<Self>) -> DispatchResult;
}

type MarketOfDisputeResolutionApi<T> = Market<
    <T as DisputeResolutionApi>::AccountId,
    <T as DisputeResolutionApi>::Balance,
    <T as DisputeResolutionApi>::BlockNumber,
    <T as DisputeResolutionApi>::Moment,
>;

pub trait DisputeResolutionApi {
    type AccountId;
    type Balance;
    type BlockNumber;
    type MarketId;
    type MaxDisputes;
    type Moment;

    /// Resolve a market.
    ///
    /// **Should** only be called if the market dispute
    /// mechanism is ready for the resolution ([`DisputeApi::on_resolution`]).
    ///
    /// # Returns
    ///
    /// Returns the consumed weight.
    fn resolve(
        market_id: &Self::MarketId,
        market: &MarketOfDisputeResolutionApi<Self>,
    ) -> Result<Weight, DispatchError>;

    /// Add a future block resolution of a disputed market.
    ///
    /// # Returns
    ///
    /// Returns the number of elements in the storage structure.
    fn add_auto_resolve(
        market_id: &Self::MarketId,
        resolve_at: Self::BlockNumber,
    ) -> Result<u32, DispatchError>;

    /// Check if a future block resolution of a disputed market exists.
    ///
    /// # Returns
    ///
    /// Returns `true` if the future block resolution exists, otherwise `false`.
    fn auto_resolve_exists(market_id: &Self::MarketId, resolve_at: Self::BlockNumber) -> bool;

    /// Remove a future block resolution of a disputed market.
    ///
    /// # Returns
    ///
    /// Returns the number of elements in the storage structure.
    fn remove_auto_resolve(market_id: &Self::MarketId, resolve_at: Self::BlockNumber) -> u32;
}
