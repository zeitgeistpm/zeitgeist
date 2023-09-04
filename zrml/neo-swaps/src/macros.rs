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

// #[cfg(test)]
// #[macro_export]
// macro_rules! balance {
//     ($x:expr) => {
//         $x * BASE
//     };
//     ($num:expr, $den:expr) => {
//         ($num * BASE) / $den
//     };
// }

#[cfg(test)]
#[macro_export]
macro_rules! assert_approx {
    ($left:expr, $right:expr, $precision:expr $(,)?) => {
        match (&$left, &$right, &$precision) {
            (left_val, right_val, precision_val) => {
                let diff = if *left_val > *right_val {
                    *left_val - *right_val
                } else {
                    *right_val - *left_val
                };
                if diff > *precision_val {
                    panic!(
                        "assertion `left approx== right` failed\n      left: {}\n     right: {}\n \
                         precision: {}\ndifference: {}",
                        *left_val, *right_val, *precision_val, diff
                    );
                }
            }
        }
    };
}
