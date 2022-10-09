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

    if is_bounded_by(value, fmin, fmax) {
        if dist(fmax, value) > dist(fmin, value) {
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
    for i in 0..max_iterations {
        mid = max.checked_add(&min).ok_or("Unexpected arithmetic error")? / 2u8.into();
        let fmid = f(mid)?;
        let size = max.checked_sub(&min).ok_or("Unexpected arithmetic error")?;
        if size < tol || fmid == value {
            iteration_count = i;
            break;
        }

        // Check on which side of `value` the preimage is located.
        if is_bounded_by(value, fmid, fmax) {
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

// Check if `t` is contained in `[x, y]` if `x <= y` or `[y, x]` if `y > x`.
fn is_bounded_by<T>(t: T, x: T, y: T) -> bool
where
    T: AtLeast32BitUnsigned + Copy,
{
    diff_sign(x, t).saturating_mul(diff_sign(y, t)) > 0
}

fn dist<T: AtLeast32BitUnsigned>(x: T, y: T) -> T {
    if x > y { x - y } else { y - x }
}
