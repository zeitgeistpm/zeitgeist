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

// TODO boost copyright

use sp_runtime::traits::AtLeast32BitUnsigned;

/// Calculate the preimage of `value` under `f` using bisection.
///
/// This function always terminates (regardless of the properties of `f`), but `f` must be
/// a monotonous function to guarantee that the result is correct. The algorithm _assumes_ that `f`
/// is monotonous, but we cannot ensure this. Calling with a non-monotonous function is undefined
/// behavior. We do some sanity checks, but in the end, the consumer is responsible for providing a
/// function which satisfies the API.
///
/// We return `(preimage, iteration_count)`. If `[min, max]` contains no preimage, then
/// `min` or `max` is returned, depending on which is closer to the preimage. If `[min, max]`
/// contains a preimage, an approximation of a preimage is returned.
///
/// # Arguments
///
/// - `f`: The function
/// - `value`: The value of the preimage
/// - `min`: The minimum value of the preimage
/// - `max`: The maximum value of the preimage
/// - `max_iterations`: Break after this many iterations
/// - `tol`: Break if the interval is smaller than this
pub(crate) fn calc_preimage<T, F>(
    f: F,
    value: T,
    mut min: T,
    mut max: T,
    max_iterations: usize,
    tol: T,
) -> Result<(T, usize), &'static str>
where
    T: AtLeast32BitUnsigned + Copy,
    F: Fn(T) -> Result<T, &'static str>,
{
    if !(min < max) {
        return Err("Sanity check failed");
    }
    let fmin = f(min)?;
    let mut fmax = f(max)?;
    if fmin == value {
        return Ok((min, 0));
    } else if fmax == value {
        return Ok((max, 0));
    }

    if is_outside_of(value, fmin, fmax) {
        if dist(fmax, value) < dist(fmin, value) {
            return Ok((max, 0));
        } else {
            return Ok((min, 0));
        }
    }
    // At this point we can assume that there exists a preimage!

    // Defensively use `for` instead of `while` or `loop` to ensure that it breaks after
    // `max_iterations`.
    let mut mid = T::zero();
    let mut iteration_count = 0;
    for i in 1..=max_iterations {
        iteration_count = i;
        let size = max.checked_sub(&min).ok_or("Unexpected arithmetic underflow")?;
        if size < tol {
            break;
        }
        mid = max.checked_add(&min).ok_or("Arithmetic overflow")? / 2u8.into();
        let fmid = f(mid)?;
        if fmid == value {
            break;
        }

        // Check on which side of `value` the preimage is located.
        if is_outside_of(value, fmid, fmax) {
            max = mid;
            fmax = fmid;
        } else {
            min = mid;
            // We (surprisingly?) don't need this:
            // fmin = fmid;
        }
    }

    Ok((mid, iteration_count))
}

// Return the sign of the (mathematical) difference x - y.
fn diff_sign<T: AtLeast32BitUnsigned>(x: T, y: T) -> i8 {
    if x < y {
        -1
    } else if y < x {
        1
    } else {
        0
    }
}

// Check if `t` lies outside of `[x, y]` if `x <= y` or `[y, x]` if `y > x`.
fn is_outside_of<T>(t: T, x: T, y: T) -> bool
where
    T: AtLeast32BitUnsigned + Copy,
{
    diff_sign(x, t).saturating_mul(diff_sign(y, t)) > 0
}

fn dist<T: AtLeast32BitUnsigned>(x: T, y: T) -> T {
    if x > y { x - y } else { y - x }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed::{bmul, bpowi};
    use test_case::test_case;
    use zeitgeist_primitives::constants::BASE;

    const _1: u128 = BASE;
    const _2: u128 = 2 * BASE;
    const _3: u128 = 3 * BASE;
    const _4: u128 = 4 * BASE;
    const _5: u128 = 5 * BASE;
    const _6: u128 = 6 * BASE;
    const _7: u128 = 7 * BASE;
    const _8: u128 = 8 * BASE;
    const _9: u128 = 9 * BASE;
    const _10: u128 = 10 * BASE;
    const _1_2: u128 = 1 * BASE / 2;
    const _3_4: u128 = 3 * BASE / 4;
    const _1_1000: u128 = BASE / 1_000;

    // Macro for comparing fixed point u128.
    #[allow(unused_macros)]
    macro_rules! assert_approx {
        ($left:expr, $right:expr, $precision:expr $(,)?) => {
            match (&$left, &$right, &$precision) {
                (left_val, right_val, precision_val) => {
                    let diff = if *left_val > *right_val {
                        *left_val - *right_val
                    } else {
                        *right_val - *left_val
                    };
                    if diff > $precision {
                        panic!("{} is not {}-close to {}", *left_val, *precision_val, *right_val);
                    }
                }
            }
        };
    }

    #[test_case(_1, _3)]
    fn calc_preimage_works_with_increasing_polynomial(value: u128, expected: u128) {
        // f(x) = 2x^3 - x^2 - x + 1 is positive and increasing on [1, \infty].
        let f = |x: u128| {
            let third_order = bmul(_2, bpowi(x, 3)?)?;
            let second_order = bpowi(x, 2)?;
            // Add positive terms first to prevent underflow.
            let result = third_order.checked_add(_1).ok_or("1")?;
            result.checked_sub(second_order.checked_add(x).ok_or("2")?).ok_or("3")
        };
        let tolerance = _1_1000;
        let (preimage, _) = calc_preimage(f, value, _3, _7, usize::MAX, _1_1000).unwrap();
        assert_approx!(preimage, expected, tolerance);
    }

    #[test]
    fn calc_preimage_breaks_after_max_iterations() {
        let f = |x: u128| Ok(x);
        let max_iterations = 1;
        let (preimage, iteration_count) = calc_preimage(f, _7, _5, _10, max_iterations, _1_1000).unwrap();
        assert_eq!(preimage, _7 + _1_2);
        assert_eq!(iteration_count, max_iterations);
    }

    #[test]
    fn calc_preimage_breaks_when_tolerance_is_violated() {
        let f = |x: u128| Ok(x);
        let (preimage, iteration_count) = calc_preimage(f, _9 - 1, _5, _9, 10, _3_4).unwrap();
        assert_eq!(preimage, _8 + _1_2);
        assert_eq!(iteration_count, 4);
    }

    #[test_case(_9, _9)]
    #[test_case(_9, _8)]
    fn calc_preimage_errors_if_range_has_no_volume(min: u128, max: u128) {
        let f = |x: u128| Ok(x);
        assert!(calc_preimage(f, _5 - 1, min, max, 10, _3_4).is_err());
    }

    #[test_case(2, 3, 1)]
    #[test_case(4, 1, 3)]
    #[test_case(5, 5, 0)]
    fn test_dist(x: u32, y: u32, expected: u32) {
        assert_eq!(dist(x, y), expected);
    }

    #[test_case(1, 5, 9, true)]
    #[test_case(5, 5, 9, false)]
    #[test_case(7, 5, 9, false)]
    #[test_case(9, 5, 9, false)]
    #[test_case(u32::MAX, 5, 9, true)]
    #[test_case(1, 9, 5, true)]
    #[test_case(5, 9, 5, false)]
    #[test_case(7, 9, 5, false)]
    #[test_case(9, 9, 5, false)]
    #[test_case(u32::MAX, 9, 5, true)]
    fn test_is_outside_of(t: u32, x: u32, y: u32, expected: bool) {
        assert_eq!(is_outside_of(t, x, y), expected);
    }
}
