// Copyright 2023-2025 Forecasting Technologies LTD.
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
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//     Copyright (c) 2019 Alain Brenzikofer, modified by GalacticCouncil(2021)
//
//     Licensed under the Apache License, Version 2.0 (the "License");
//     you may not use this file except in compliance with the License.
//     You may obtain a copy of the License at
//
//          http://www.apache.org/licenses/LICENSE-2.0
//
//     Unless required by applicable law or agreed to in writing, software
//     distributed under the License is distributed on an "AS IS" BASIS,
//     WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//     See the License for the specific language governing permissions and
//     limitations under the License.
//
//     Original source: https://github.com/encointer/substrate-fixed
//
// The changes applied are: Re-used and extended tests for `exp` and other
// functions.

pub(crate) use hydra_dx_math::transcendental::{exp, ln};

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::str::FromStr;
    use fixed::types::U64F64;
    use test_case::test_case;

    type S = U64F64;
    type D = U64F64;

    #[test_case("0", false, "1")]
    #[test_case("0", true, "1")]
    #[test_case("1", false, "2.7182818284590452353")]
    #[test_case("1", true, "0.367879441171442321595523770161460867445")]
    #[test_case("2", false, "7.3890560989306502265")]
    #[test_case("2", true, "0.13533528323661269186")]
    #[test_case("0.1", false, "1.1051709180756476246")]
    #[test_case("0.1", true, "0.9048374180359595733")]
    #[test_case("0.9", false, "2.4596031111569496633")]
    #[test_case("0.9", true, "0.40656965974059911195")]
    #[test_case("1.5", false, "4.481689070338064822")]
    #[test_case("1.5", true, "0.22313016014842982894")]
    #[test_case("3.3", false, "27.1126389206578874259")]
    #[test_case("3.3", true, "0.03688316740124000543")]
    #[test_case("7.3456", false, "1549.3643050275008503592")]
    #[test_case("7.3456", true, "0.00064542599616831253")]
    #[test_case("12.3456789", false, "229964.194569082134542849")]
    #[test_case("12.3456789", true, "0.00000434850304358833")]
    #[test_case("13", false, "442413.39200892050332603603")]
    #[test_case("13", true, "0.0000022603294069810542")]
    fn exp_works(operand: &str, neg: bool, expected: &str) {
        let o = U64F64::from_str(operand).unwrap();
        let e = U64F64::from_str(expected).unwrap();
        assert_eq!(exp::<S, D>(o, neg).unwrap(), e);
    }

    #[test_case("1", "0", false)]
    #[test_case("2", "0.69314718055994530943", false)]
    #[test_case("3", "1.09861228866810969136", false)]
    #[test_case("2.718281828459045235360287471352662497757", "1", false)]
    #[test_case("1.1051709180756476246", "0.09999999999999999975", false)]
    #[test_case("2.4596031111569496633", "0.89999999999999999976", false)]
    #[test_case("4.481689070338064822", "1.49999999999999999984", false)]
    #[test_case("27.1126389206578874261", "3.3", false)]
    #[test_case("1549.3643050275008503592", "7.34560000000000000003", false)]
    #[test_case("229964.194569082134542849", "12.3456789000000000002", false)]
    #[test_case("442413.39200892050332603603", "13.0000000000000000002", false)]
    #[test_case("0.9048374180359595733", "0.09999999999999999975", true)]
    #[test_case("0.40656965974059911195", "0.8999999999999999998", true)]
    #[test_case("0.22313016014842982894", "1.4999999999999999999", true)]
    #[test_case("0.03688316740124000543", "3.3000000000000000005", true)]
    #[test_case("0.00064542599616831253", "7.34560000000000002453", true)]
    #[test_case("0.00000434850304358833", "12.34567890000000711117", true)]
    #[test_case("0.0000022603294069810542", "13.0000000000000045352", true)]
    #[test_case("1.0001", "0.00009999500033330827", false)]
    #[test_case("1.00000001", "0.0000000099999999499", false)]
    #[test_case("0.9999", "0.00010000500033335825", true)]
    #[test_case("0.99999999", "0.00000001000000004987", true)]
    // Powers of 2 (since we're using squares when calculating the fractional part of log2.
    #[test_case("3.999999999", "1.38629436086989061877", false)]
    #[test_case("4", "1.38629436111989061886", false)]
    #[test_case("4.000000001", "1.3862943613698906188", false)]
    #[test_case("7.999999999", "2.07944154155483592824", false)]
    #[test_case("8", "2.0794415416798359283", false)]
    #[test_case("8.000000001", "2.0794415418048359282", false)]
    #[test_case("0.499999999", "0.69314718255994531136", true)]
    #[test_case("0.5", "0.69314718055994530943", true)]
    #[test_case("0.500000001", "0.69314717855994531135", true)]
    #[test_case("0.249999999", "1.38629436511989062684", true)]
    #[test_case("0.25", "1.38629436111989061886", true)]
    #[test_case("0.250000001", "1.38629435711989062676", true)]
    fn ln_works(operand: &str, expected_abs: &str, expected_neg: bool) {
        let o = U64F64::from_str(operand).unwrap();
        let e = U64F64::from_str(expected_abs).unwrap();
        let (a, n) = ln::<S, D>(o).unwrap();
        assert_eq!(a, e);
        assert_eq!(n, expected_neg);
    }
}
