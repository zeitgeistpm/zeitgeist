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

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::pallet_prelude::DispatchError;

    const DEFAULT_ERROR: DispatchResult = Err(DispatchError::Other("unimportant"));
    const ONE: Weight = Weight::from_all(1);
    const TWO: Weight = Weight::from_all(2);
    const THREE: Weight = Weight::from_all(3);

    struct ExecutionPath;
    impl MarketTransitionApi<u128> for ExecutionPath {
        fn on_proposal(_market_id: &u128) -> ResultWithWeightInfo<DispatchResult> {
            panic!("on_proposal");
        }
        fn on_activation(_market_id: &u128) -> ResultWithWeightInfo<DispatchResult> {
            panic!("on_activation");
        }
        fn on_closure(_market_id: &u128) -> ResultWithWeightInfo<DispatchResult> {
            panic!("on_closure");
        }
        fn on_report(_market_id: &u128) -> ResultWithWeightInfo<DispatchResult> {
            panic!("on_report");
        }
        fn on_dispute(_market_id: &u128) -> ResultWithWeightInfo<DispatchResult> {
            panic!("on_dispute");
        }
        fn on_resolution(_market_id: &u128) -> ResultWithWeightInfo<DispatchResult> {
            panic!("on_resolution");
        }
    }

    struct SuccessPath;
    impl MarketTransitionApi<u128> for SuccessPath {
        fn on_proposal(_market_id: &u128) -> ResultWithWeightInfo<DispatchResult> {
            ResultWithWeightInfo::new(Ok(()), ONE)
        }
        fn on_activation(_market_id: &u128) -> ResultWithWeightInfo<DispatchResult> {
            ResultWithWeightInfo::new(Ok(()), ONE)
        }
        fn on_closure(_market_id: &u128) -> ResultWithWeightInfo<DispatchResult> {
            ResultWithWeightInfo::new(Ok(()), ONE)
        }
        fn on_report(_market_id: &u128) -> ResultWithWeightInfo<DispatchResult> {
            ResultWithWeightInfo::new(Ok(()), ONE)
        }
        fn on_dispute(_market_id: &u128) -> ResultWithWeightInfo<DispatchResult> {
            ResultWithWeightInfo::new(Ok(()), ONE)
        }
        fn on_resolution(_market_id: &u128) -> ResultWithWeightInfo<DispatchResult> {
            ResultWithWeightInfo::new(Ok(()), ONE)
        }
    }

    struct FailurePath;
    impl MarketTransitionApi<u128> for FailurePath {
        fn on_proposal(_market_id: &u128) -> ResultWithWeightInfo<DispatchResult> {
            ResultWithWeightInfo::new(DEFAULT_ERROR, TWO)
        }
        fn on_activation(_market_id: &u128) -> ResultWithWeightInfo<DispatchResult> {
            ResultWithWeightInfo::new(DEFAULT_ERROR, TWO)
        }
        fn on_closure(_market_id: &u128) -> ResultWithWeightInfo<DispatchResult> {
            ResultWithWeightInfo::new(DEFAULT_ERROR, TWO)
        }
        fn on_report(_market_id: &u128) -> ResultWithWeightInfo<DispatchResult> {
            ResultWithWeightInfo::new(DEFAULT_ERROR, TWO)
        }
        fn on_dispute(_market_id: &u128) -> ResultWithWeightInfo<DispatchResult> {
            ResultWithWeightInfo::new(DEFAULT_ERROR, TWO)
        }
        fn on_resolution(_market_id: &u128) -> ResultWithWeightInfo<DispatchResult> {
            ResultWithWeightInfo::new(DEFAULT_ERROR, TWO)
        }
    }

    #[test]
    #[should_panic(expected = "on_proposal")]
    fn correct_execution_path_for_tuples_on_proposal() {
        <(ExecutionPath,)>::on_proposal(&0);
    }

    #[test]
    #[should_panic(expected = "on_activation")]
    fn correct_execution_path_for_tuples_on_activation() {
        <(ExecutionPath,)>::on_activation(&0);
    }

    #[test]
    #[should_panic(expected = "on_closure")]
    fn correct_execution_path_for_tuples_on_closure() {
        <(ExecutionPath,)>::on_closure(&0);
    }

    #[test]
    #[should_panic(expected = "on_report")]
    fn correct_execution_path_for_tuples_on_report() {
        <(ExecutionPath,)>::on_report(&0);
    }

    #[test]
    #[should_panic(expected = "on_dispute")]
    fn correct_execution_path_for_tuples_on_dispute() {
        <(ExecutionPath,)>::on_dispute(&0);
    }

    #[test]
    #[should_panic(expected = "on_resolution")]
    fn correct_execution_path_for_tuples_on_resolution() {
        <(ExecutionPath,)>::on_resolution(&0);
    }

    #[test]
    fn provides_correct_result_on_proposal() {
        let mut result = <(SuccessPath,)>::on_proposal(&0);
        assert_eq!(result.result, Ok(()));
        assert_eq!(result.weight, ONE);

        result = <(SuccessPath, FailurePath)>::on_proposal(&0);
        assert_eq!(result.result, DEFAULT_ERROR);
        assert_eq!(result.weight, THREE);

        result = <(FailurePath, SuccessPath)>::on_proposal(&0);
        assert_eq!(result.result, DEFAULT_ERROR);
        assert_eq!(result.weight, TWO);
    }

    #[test]
    fn provides_correct_result_on_activation() {
        let mut result = <(SuccessPath,)>::on_activation(&0);
        assert_eq!(result.result, Ok(()));
        assert_eq!(result.weight, ONE);

        result = <(SuccessPath, FailurePath)>::on_activation(&0);
        assert_eq!(result.result, DEFAULT_ERROR);
        assert_eq!(result.weight, THREE);

        result = <(FailurePath, SuccessPath)>::on_activation(&0);
        assert_eq!(result.result, DEFAULT_ERROR);
        assert_eq!(result.weight, TWO);
    }

    #[test]
    fn provides_correct_result_on_closure() {
        let mut result = <(SuccessPath,)>::on_closure(&0);
        assert_eq!(result.result, Ok(()));
        assert_eq!(result.weight, ONE);

        result = <(SuccessPath, FailurePath)>::on_closure(&0);
        assert_eq!(result.result, DEFAULT_ERROR);
        assert_eq!(result.weight, THREE);

        result = <(FailurePath, SuccessPath)>::on_closure(&0);
        assert_eq!(result.result, DEFAULT_ERROR);
        assert_eq!(result.weight, TWO);
    }

    #[test]
    fn provides_correct_result_on_report() {
        let mut result = <(SuccessPath,)>::on_report(&0);
        assert_eq!(result.result, Ok(()));
        assert_eq!(result.weight, ONE);

        result = <(SuccessPath, FailurePath)>::on_report(&0);
        assert_eq!(result.result, DEFAULT_ERROR);
        assert_eq!(result.weight, THREE);

        result = <(FailurePath, SuccessPath)>::on_report(&0);
        assert_eq!(result.result, DEFAULT_ERROR);
        assert_eq!(result.weight, TWO);
    }

    #[test]
    fn provides_correct_result_on_dispute() {
        let mut result = <(SuccessPath,)>::on_dispute(&0);
        assert_eq!(result.result, Ok(()));
        assert_eq!(result.weight, ONE);

        result = <(SuccessPath, FailurePath)>::on_dispute(&0);
        assert_eq!(result.result, DEFAULT_ERROR);
        assert_eq!(result.weight, THREE);

        result = <(FailurePath, SuccessPath)>::on_dispute(&0);
        assert_eq!(result.result, DEFAULT_ERROR);
        assert_eq!(result.weight, TWO);
    }

    #[test]
    fn provides_correct_result_on_resolution() {
        let mut result = <(SuccessPath,)>::on_resolution(&0);
        assert_eq!(result.result, Ok(()));
        assert_eq!(result.weight, ONE);

        result = <(SuccessPath, FailurePath)>::on_resolution(&0);
        assert_eq!(result.result, DEFAULT_ERROR);
        assert_eq!(result.weight, THREE);

        result = <(FailurePath, SuccessPath)>::on_resolution(&0);
        assert_eq!(result.result, DEFAULT_ERROR);
        assert_eq!(result.weight, TWO);
    }
}
