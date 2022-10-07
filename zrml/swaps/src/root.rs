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

use sp_runtime::traits::AtLeast32BitUnsigned;

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

fn dist<T: AtLeast32BitUnsigned>(x: T, y: T) -> T {
    if x > y { x - y } else { y - x }
}

// Calculate the preimage of `value` under `f`.
//
// Requirements for function: None
//
// Converges to correct value if:
// - f(min) and f(max) are not on the same side of value
// - f is either strictly increasing or decreasing
//
// The algorithm _assumes_ that `f` is monotonous, but we cannot ensure this. Calling with
// a non-monotonous function is undefined behavior.
// We do some sanity checks, but in the end, the consumer is responsible for providing a function
// which satisfies the API.
//
// Note: If there is no preimage of `value`, we take the next best value.
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
    F: Fn(T) -> T,
{
    if !(min < max) {
        return Err("Sanity check failed");
    }
    let mut fmin = f(min);
    let mut fmax = f(max);
    if fmin == value {
        return Ok((min, 0));
    } else if fmax == value {
        return Ok((max, 0));
    }
    // If there is no preimage available, we take the next best boundary value.
    if diff_sign(fmin, value) * diff_sign(fmax, value) > 0 {
        if dist(fmax, value) > dist(fmin, value) {
            return Ok((max, 0));
        } else {
            return Ok((min, 0));
        }
    }

    // Defensively use `for` instead of `while` or `loop` to ensure that it breaks after
    // `max_iterations`.
    let mut mid = T::zero();
    let mut iterations = 0;
    for i in 0..max_iterations {
        mid = (max + min) / 2u8.into();
        let size = max.checked_sub(&min).ok_or("Unexpected arithmetic error")?;
        if size < tol {
            iterations = i;
            break;
        }

        let fmid = f(mid);
        if fmid == value {
            iterations = i;
            break;
        }
        // Proceed with [min, mid] if mid and max are located on the same side of value; [mid, max]
        // otherwise.
        if diff_sign(fmid, value) * diff_sign(fmax, value) > 0 {
            max = mid;
            fmax = fmid;
        } else {
            min = mid;
            fmin = fmid;
        }
    }

    Ok((mid, iterations))
}

// TODO Put checks in the consumer:
//
// - New value is closer to 1 than before
// - Max iterations not reached
// - Check that expected number of iterations was not violated
