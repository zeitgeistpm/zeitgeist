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
//
// This file incorporates work covered by the license above but
// published without copyright notice by Balancer Labs
// (<https://balancer.finance>, contact@balancer.finance) in the
// balancer-core repository
// <https://github.com/balancer-labs/balancer-core>.

use frame_support::dispatch::DispatchError;
use zeitgeist_primitives::{
    constants::BASE,
    math::{
        checked_ops_res::{CheckedAddRes, CheckedDivRes, CheckedMulRes, CheckedSubRes},
        fixed::{FixedDiv, FixedMul},
    },
};

/// The amount of precision to use in exponentiation.
pub const BPOW_PRECISION: u128 = 10;
/// The minimum value of the base parameter in bpow_approx.
pub const BPOW_APPROX_BASE_MIN: u128 = BASE / 4;
/// The maximum value of the base parameter in bpow_approx.
pub const BPOW_APPROX_BASE_MAX: u128 = 7 * BASE / 4;
/// The maximum number of terms from the binomial series used to calculate bpow_approx.
pub const BPOW_APPROX_MAX_ITERATIONS: u128 = 100;

pub fn btoi(a: u128) -> Result<u128, DispatchError> {
    a.checked_div_res(&BASE)
}

pub fn bfloor(a: u128) -> Result<u128, DispatchError> {
    btoi(a)?.checked_mul_res(&BASE)
}

pub fn bsub_sign(a: u128, b: u128) -> Result<(u128, bool), DispatchError> {
    Ok(if a >= b { (a.checked_sub_res(&b)?, false) } else { (b.checked_sub_res(&a)?, true) })
}

pub fn bpowi(a: u128, n: u128) -> Result<u128, DispatchError> {
    let mut z = if n % 2 != 0 { a } else { BASE };

    let mut b = a;
    let mut m = n.checked_div_res(&2)?;

    while m != 0 {
        b = b.bmul(b)?;

        if m % 2 != 0 {
            z = z.bmul(b)?;
        }

        m = m.checked_div_res(&2)?;
    }

    Ok(z)
}

/// Compute the power `base ** exp`.
///
/// # Arguments
///
/// * `base`: The base, a number between `BASE / 4` and `7 * BASE / 4`
/// * `exp`: The exponent
///
/// # Errors
///
/// If this function encounters an arithmetic over/underflow, or if the numerical limits
/// for `base` (specified above) are violated, a `DispatchError::Other` is returned.
pub fn bpow(base: u128, exp: u128) -> Result<u128, DispatchError> {
    let whole = bfloor(exp)?;
    let remain = exp.checked_sub_res(&whole)?;

    let whole_pow = bpowi(base, btoi(whole)?)?;

    if remain == 0 {
        return Ok(whole_pow);
    }

    let partial_result = bpow_approx(base, remain)?;
    whole_pow.bmul(partial_result)
}

/// Compute an estimate of the power `base ** exp`.
///
/// # Arguments
///
/// * `base`: The base, an element of `[BASE / 4, 7 * BASE / 4]`
/// * `exp`: The exponent, an element of `[0, BASE]`
///
/// # Errors
///
/// If this function encounters an arithmetic over/underflow, or if the numerical limits
/// for `base` or `exp` (specified above) are violated, a `DispatchError::Other` is
/// returned.
pub fn bpow_approx(base: u128, exp: u128) -> Result<u128, DispatchError> {
    // We use the binomial power series for this calculation. We stop adding terms to
    // the result as soon as one term is smaller than `BPOW_PRECISION`. (Thanks to the
    // limits on `base` and `exp`, this means that the total error should not exceed
    // 4*BPOW_PRECISION`.)
    if exp > BASE {
        return Err(DispatchError::Other("[bpow_approx]: expected exp <= BASE"));
    }
    if base < BPOW_APPROX_BASE_MIN {
        return Err(DispatchError::Other("[bpow_approx]: expected base >= BASE / 4"));
    }
    if base > BPOW_APPROX_BASE_MAX {
        return Err(DispatchError::Other("[bpow_approx]: expected base <= 7 * BASE / 4"));
    }

    let a = exp;
    let (x, xneg) = bsub_sign(base, BASE)?;
    let mut term = BASE;
    let mut sum = term;
    let mut negative = false;

    // term(k) = numer / denom
    //         = (product(a - i - 1, i=1-->k) * x^k) / (k!)
    // each iteration, multiply previous term by (a-(k-1)) * x / k
    // continue until term is less than precision
    for i in 1..=BPOW_APPROX_MAX_ITERATIONS {
        if term < BPOW_PRECISION {
            break;
        }

        let big_k = i.checked_mul_res(&BASE)?;
        let (c, cneg) = bsub_sign(a, big_k.checked_sub_res(&BASE)?)?;
        term = term.bmul(c.bmul(x)?)?;
        term = term.bdiv(big_k)?;
        if term == 0 {
            break;
        }

        if xneg {
            negative = !negative;
        }
        if cneg {
            negative = !negative;
        }
        if negative {
            // Never underflows. In fact, the absolute value of the terms is strictly
            // decreasing thanks to the numerical limits.
            sum = sum.checked_sub_res(&term)?;
        } else {
            sum = sum.checked_add_res(&term)?;
        }
    }

    // If term is still large, then MAX_ITERATIONS was violated (can't happen with the current
    // limits).
    if term >= BPOW_PRECISION {
        return Err(DispatchError::Other("[bpow_approx] Maximum number of iterations exceeded"));
    }

    Ok(sum)
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{assert_err, dispatch::DispatchError};
    use more_asserts::assert_le;

    #[test]
    fn bpow_has_minimum_set_of_correct_values() {
        let test_vector: Vec<(u128, u128, u128)> = vec![
            (2500000000, 0, 10000000000),
            (2500000000, 10000000000, 2500000000),
            (2500000000, 33333333333, 98431332),
            (2500000000, 200000000, 9726549474),
            (2500000000, 500000000000, 0),
            (5000000000, 0, 10000000000),
            (5000000000, 10000000000, 5000000000),
            (5000000000, 33333333333, 992125657),
            (5000000000, 200000000, 9862327044),
            (5000000000, 500000000000, 0),
            (7500000000, 0, 10000000000),
            (7500000000, 10000000000, 7500000000),
            (7500000000, 33333333333, 3832988750),
            (7500000000, 200000000, 9942628790),
            (7500000000, 500000000000, 5663),
            (10000000000, 0, 10000000000),
            (10000000000, 10000000000, 10000000000),
            (10000000000, 33333333333, 10000000000),
            (10000000000, 200000000, 10000000000),
            (10000000000, 500000000000, 10000000000),
            (12500000000, 0, 10000000000),
            (12500000000, 10000000000, 12500000000),
            (12500000000, 33333333333, 21039401269),
            (12500000000, 200000000, 10044728444),
            (12500000000, 500000000000, 700649232162408),
            (15000000000, 0, 10000000000),
            (15000000000, 10000000000, 15000000000),
            (15000000000, 33333333333, 38634105686),
            (15000000000, 200000000, 10081422716),
            (15000000000, 500000000000, 6376215002140495869),
            (17500000000, 0, 10000000000),
            (17500000000, 10000000000, 17500000000),
            (17500000000, 33333333333, 64584280985),
            (17500000000, 200000000, 10112551840),
            (17500000000, 500000000000, 14187387615511831479362),
        ];
        for (base, exp, expected) in test_vector.iter() {
            let result = bpow(*base, *exp).unwrap();
            let precision = *expected / BASE + 4 * BPOW_PRECISION; // relative + absolute error
            let diff = if result > *expected { result - *expected } else { *expected - result };
            assert_le!(diff, precision);
        }
    }

    #[test]
    fn bpow_returns_error_when_parameters_are_outside_of_specified_limits() {
        let test_vector: Vec<(u128, u128)> =
            vec![(BASE / 10, 3 * BASE / 2), (2 * BASE - BASE / 10, 3 * BASE / 2)];
        for (base, exp) in test_vector.iter() {
            assert!(bpow(*base, *exp).is_err());
        }
    }

    #[test]
    fn bpow_approx_has_minimum_set_of_correct_values() {
        let precision = 4 * BPOW_PRECISION;
        let test_vector: Vec<(u128, u128, u128)> = vec![
            (2500000000, 0, 10000000000),
            (2500000000, 1000000000, 8705505632),
            (2500000000, 2000000000, 7578582832),
            (2500000000, 3000000000, 6597539553),
            (2500000000, 4000000000, 5743491774),
            (2500000000, 5000000000, 5000000000),
            (2500000000, 6000000000, 4352752816),
            (2500000000, 7000000000, 3789291416),
            (2500000000, 8000000000, 3298769776),
            (2500000000, 9000000000, 2871745887),
            (2500000000, 10000000000, 2500000000),
            (5000000000, 0, 10000000000),
            (5000000000, 1000000000, 9330329915),
            (5000000000, 2000000000, 8705505632),
            (5000000000, 3000000000, 8122523963),
            (5000000000, 4000000000, 7578582832),
            (5000000000, 5000000000, 7071067811),
            (5000000000, 6000000000, 6597539553),
            (5000000000, 7000000000, 6155722066),
            (5000000000, 8000000000, 5743491774),
            (5000000000, 9000000000, 5358867312),
            (5000000000, 10000000000, 5000000000),
            (7500000000, 0, 10000000000),
            (7500000000, 1000000000, 9716416578),
            (7500000000, 2000000000, 9440875112),
            (7500000000, 3000000000, 9173147546),
            (7500000000, 4000000000, 8913012289),
            (7500000000, 5000000000, 8660254037),
            (7500000000, 6000000000, 8414663590),
            (7500000000, 7000000000, 8176037681),
            (7500000000, 8000000000, 7944178807),
            (7500000000, 9000000000, 7718895067),
            (7500000000, 10000000000, 7500000000),
            (10000000000, 0, 10000000000),
            (10000000000, 1000000000, 10000000000),
            (10000000000, 2000000000, 10000000000),
            (10000000000, 3000000000, 10000000000),
            (10000000000, 4000000000, 10000000000),
            (10000000000, 5000000000, 10000000000),
            (10000000000, 6000000000, 10000000000),
            (10000000000, 7000000000, 10000000000),
            (10000000000, 8000000000, 10000000000),
            (10000000000, 9000000000, 10000000000),
            (10000000000, 10000000000, 10000000000),
            (12500000000, 0, 10000000000),
            (12500000000, 1000000000, 10225651825),
            (12500000000, 2000000000, 10456395525),
            (12500000000, 3000000000, 10692345999),
            (12500000000, 4000000000, 10933620739),
            (12500000000, 5000000000, 11180339887),
            (12500000000, 6000000000, 11432626298),
            (12500000000, 7000000000, 11690605597),
            (12500000000, 8000000000, 11954406247),
            (12500000000, 9000000000, 12224159606),
            (12500000000, 10000000000, 12500000000),
            (15000000000, 0, 10000000000),
            (15000000000, 1000000000, 10413797439),
            (15000000000, 2000000000, 10844717711),
            (15000000000, 3000000000, 11293469354),
            (15000000000, 4000000000, 11760790225),
            (15000000000, 5000000000, 12247448713),
            (15000000000, 6000000000, 12754245006),
            (15000000000, 7000000000, 13282012399),
            (15000000000, 8000000000, 13831618672),
            (15000000000, 9000000000, 14403967511),
            (15000000000, 10000000000, 15000000000),
            (17500000000, 0, 10000000000),
            (17500000000, 1000000000, 10575570503),
            (17500000000, 2000000000, 11184269147),
            (17500000000, 3000000000, 11828002689),
            (17500000000, 4000000000, 12508787635),
            (17500000000, 5000000000, 13228756555),
            (17500000000, 6000000000, 13990164762),
            (17500000000, 7000000000, 14795397379),
            (17500000000, 8000000000, 15646976811),
            (17500000000, 9000000000, 16547570643),
            (17500000000, 10000000000, 17500000000),
        ];
        for (base, exp, expected) in test_vector.iter() {
            let result = bpow_approx(*base, *exp).unwrap();
            let diff = if result > *expected { result - *expected } else { *expected - result };
            assert_le!(diff, precision);
        }
    }

    #[test]
    fn bpow_approx_returns_error_when_parameters_are_outside_of_specified_limits() {
        let test_vector: Vec<(u128, u128, DispatchError)> = vec![
            (BASE, BASE + 1, DispatchError::Other("[bpow_approx]: expected exp <= BASE")),
            (BASE / 10, BASE / 2, DispatchError::Other("[bpow_approx]: expected base >= BASE / 4")),
            (
                2 * BASE - BASE / 10,
                BASE / 2,
                DispatchError::Other("[bpow_approx]: expected base <= 7 * BASE / 4"),
            ),
        ];
        for (base, exp, err) in test_vector.iter() {
            assert_err!(bpow_approx(*base, *exp), *err);
        }
    }
}
