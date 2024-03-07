// Copyright 2024 Forecasting Technologies LTD.
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

use crate::types::ResultWithWeightInfo;
use frame_support::pallet_prelude::{DispatchResult, Weight};

/// API that is used to catch market state transitions.
pub trait MarketTransitionApi<MI> {
    fn on_proposal(_market_id: &MI) -> ResultWithWeightInfo<DispatchResult> {
        ResultWithWeightInfo::new(Ok(()), Weight::zero())
    }
    fn on_activation(_market_id: &MI) -> ResultWithWeightInfo<DispatchResult> {
        ResultWithWeightInfo::new(Ok(()), Weight::zero())
    }
    fn on_closure(_market_id: &MI) -> ResultWithWeightInfo<DispatchResult> {
        ResultWithWeightInfo::new(Ok(()), Weight::zero())
    }
    fn on_report(_market_id: &MI) -> ResultWithWeightInfo<DispatchResult> {
        ResultWithWeightInfo::new(Ok(()), Weight::zero())
    }
    fn on_dispute(_market_id: &MI) -> ResultWithWeightInfo<DispatchResult> {
        ResultWithWeightInfo::new(Ok(()), Weight::zero())
    }
    fn on_resolution(_market_id: &MI) -> ResultWithWeightInfo<DispatchResult> {
        ResultWithWeightInfo::new(Ok(()), Weight::zero())
    }
}

#[impl_trait_for_tuples::impl_for_tuples(8)]
#[allow(clippy::let_and_return)]
/// Implementation returns on first error or after successful execution of all elements.
impl<MI> MarketTransitionApi<MI> for Tuple {
    fn on_proposal(market_id: &MI) -> ResultWithWeightInfo<DispatchResult> {
        let mut collective_result = ResultWithWeightInfo::new(Ok(()), Weight::zero());
        for_tuples!( #(
            let result = Tuple::on_proposal(market_id);
            collective_result.result = result.result;
            collective_result.weight = collective_result.weight.saturating_add(result.weight);
            if collective_result.result.is_err() {
                return collective_result;
            }
        )* );
        collective_result
    }
    fn on_activation(market_id: &MI) -> ResultWithWeightInfo<DispatchResult> {
        let mut collective_result = ResultWithWeightInfo::new(Ok(()), Weight::zero());
        for_tuples!( #(
            let result = Tuple::on_activation(market_id);
            collective_result.result = result.result;
            collective_result.weight = collective_result.weight.saturating_add(result.weight);
            if collective_result.result.is_err() {
                return collective_result;
            }
        )* );
        collective_result
    }
    fn on_closure(market_id: &MI) -> ResultWithWeightInfo<DispatchResult> {
        let mut collective_result = ResultWithWeightInfo::new(Ok(()), Weight::zero());
        for_tuples!( #(
            let result = Tuple::on_closure(market_id);
            collective_result.result = result.result;
            collective_result.weight = collective_result.weight.saturating_add(result.weight);
            if collective_result.result.is_err() {
                return collective_result;
            }
        )* );
        collective_result
    }
    fn on_report(market_id: &MI) -> ResultWithWeightInfo<DispatchResult> {
        let mut collective_result = ResultWithWeightInfo::new(Ok(()), Weight::zero());
        for_tuples!( #(
            let result = Tuple::on_report(market_id);
            collective_result.result = result.result;
            collective_result.weight = collective_result.weight.saturating_add(result.weight);
            if collective_result.result.is_err() {
                return collective_result;
            }
        )* );
        collective_result
    }
    fn on_dispute(market_id: &MI) -> ResultWithWeightInfo<DispatchResult> {
        let mut collective_result = ResultWithWeightInfo::new(Ok(()), Weight::zero());
        for_tuples!( #(
            let result = Tuple::on_dispute(market_id);
            collective_result.result = result.result;
            collective_result.weight = collective_result.weight.saturating_add(result.weight);
            if collective_result.result.is_err() {
                return collective_result;
            }
        )* );
        collective_result
    }
    fn on_resolution(market_id: &MI) -> ResultWithWeightInfo<DispatchResult> {
        let mut collective_result = ResultWithWeightInfo::new(Ok(()), Weight::zero());
        for_tuples!( #(
            let result = Tuple::on_resolution(market_id);
            collective_result.result = result.result;
            collective_result.weight = collective_result.weight.saturating_add(result.weight);
            if collective_result.result.is_err() {
                return collective_result;
            }
        )* );
        collective_result
    }
}
