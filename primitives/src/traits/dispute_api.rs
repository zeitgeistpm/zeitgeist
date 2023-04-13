// Copyright 2023 Forecasting Technologies LTD.
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

extern crate alloc;

use crate::{
    outcome_report::OutcomeReport,
    types::{Asset, GlobalDisputeItem, Market},
};
use alloc::vec::Vec;
use frame_support::{dispatch::DispatchResult, pallet_prelude::Weight};
use parity_scale_codec::MaxEncodedLen;
use sp_runtime::DispatchError;

// Abstraction of the market type, which is not a part of `DisputeApi` because Rust doesn't support
// type aliases in traits.
type MarketOfDisputeApi<T> = Market<
    <T as DisputeApi>::AccountId,
    <T as DisputeApi>::Balance,
    <T as DisputeApi>::BlockNumber,
    <T as DisputeApi>::Moment,
    Asset<<T as DisputeApi>::MarketId>,
>;

type GlobalDisputeItemOfDisputeApi<T> =
    GlobalDisputeItem<<T as DisputeApi>::AccountId, <T as DisputeApi>::Balance>;

pub trait DisputeApi {
    type AccountId;
    type Balance;
    type NegativeImbalance;
    type BlockNumber;
    type MarketId: MaxEncodedLen;
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
    fn on_resolution(
        market_id: &Self::MarketId,
        market: &MarketOfDisputeApi<Self>,
    ) -> Result<Option<OutcomeReport>, DispatchError>;

    /// Allow the transfer of funds from the API caller to the API consumer and back.
    /// This can be based on the final resolution outcome of the market.
    /// **May** assume that `market.dispute_mechanism` refers to the calling dispute API.
    ///
    /// # Arguments
    /// * `market_id` - The identifier of the market.
    /// * `market` - The market data.
    /// * `resolved_outcome` - The final resolution outcome of the market.
    /// * `amount` - The amount of funds transferred to the dispute mechanism.
    ///
    /// # Returns
    /// Returns a negative imbalance in order to transfer funds back to the caller.
    fn exchange(
        market_id: &Self::MarketId,
        market: &MarketOfDisputeApi<Self>,
        resolved_outcome: &OutcomeReport,
        amount: Self::NegativeImbalance,
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
    ///
    /// # Returns
    /// Returns the initial vote outcomes with initial vote value and owner of the vote.
    fn on_global_dispute(
        market_id: &Self::MarketId,
        market: &MarketOfDisputeApi<Self>,
    ) -> Result<Vec<GlobalDisputeItemOfDisputeApi<Self>>, DispatchError>;

    /// Allow the API consumer to clear storage items of the dispute mechanism.
    /// This may be called, when the dispute mechanism is no longer needed.
    /// **May** assume that `market.dispute_mechanism` refers to the calling dispute API.
    fn clear(market_id: &Self::MarketId, market: &MarketOfDisputeApi<Self>) -> DispatchResult;
}

type MarketOfDisputeResolutionApi<T> = Market<
    <T as DisputeResolutionApi>::AccountId,
    <T as DisputeResolutionApi>::Balance,
    <T as DisputeResolutionApi>::BlockNumber,
    <T as DisputeResolutionApi>::Moment,
    Asset<<T as DisputeResolutionApi>::MarketId>,
>;

pub trait DisputeResolutionApi {
    type AccountId;
    type Balance;
    type BlockNumber;
    type MarketId: MaxEncodedLen;
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
