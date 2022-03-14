use crate::{
    check_arithm_rslt::CheckArithmRslt,
    consts::{
        BPOW_APPROX_BASE_MAX, BPOW_APPROX_BASE_MIN, BPOW_APPROX_MAX_ITERATIONS, BPOW_PRECISION,
    },
};
use frame_support::dispatch::DispatchError;
use zeitgeist_primitives::constants::BASE;

pub fn btoi(a: u128) -> Result<u128, DispatchError> {
    a.check_div_rslt(&BASE)
}

pub fn bfloor(a: u128) -> Result<u128, DispatchError> {
    btoi(a)?.check_mul_rslt(&BASE)
}

pub fn bsub_sign(a: u128, b: u128) -> Result<(u128, bool), DispatchError> {
    Ok(if a >= b { (a.check_sub_rslt(&b)?, false) } else { (b.check_sub_rslt(&a)?, true) })
}

pub fn bmul(a: u128, b: u128) -> Result<u128, DispatchError> {
    let c0 = a.check_mul_rslt(&b)?;
    let c1 = c0.check_add_rslt(&BASE.check_div_rslt(&2)?)?;
    c1.check_div_rslt(&BASE)
}

pub fn bdiv(a: u128, b: u128) -> Result<u128, DispatchError> {
    let c0 = a.check_mul_rslt(&BASE)?;
    let c1 = c0.check_add_rslt(&b.check_div_rslt(&2)?)?;
    c1.check_div_rslt(&b)
}

pub fn bpowi(a: u128, n: u128) -> Result<u128, DispatchError> {
    let mut z = if n % 2 != 0 { a } else { BASE };

    let mut b = a;
    let mut m = n.check_div_rslt(&2)?;

    while m != 0 {
        b = bmul(b, b)?;

        if m % 2 != 0 {
            z = bmul(z, b)?;
        }

        m = m.check_div_rslt(&2)?;
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
    let remain = exp.check_sub_rslt(&whole)?;

    let whole_pow = bpowi(base, btoi(whole)?)?;

    if remain == 0 {
        return Ok(whole_pow);
    }

    let partial_result = bpow_approx(base, remain)?;
    bmul(whole_pow, partial_result)
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

        let big_k = i.check_mul_rslt(&BASE)?;
        let (c, cneg) = bsub_sign(a, big_k.check_sub_rslt(&BASE)?)?;
        term = bmul(term, bmul(c, x)?)?;
        term = bdiv(term, big_k)?;
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
            sum = sum.check_sub_rslt(&term)?;
        } else {
            sum = sum.check_add_rslt(&term)?;
        }
    }

    // If term is still large, then MAX_ITERATIONS was violated (can't happen with the current
    // limits).
    if term >= BPOW_PRECISION {
        return Err(DispatchError::Other(
            "[bpow_approx] Maximum number of iterations exceeded",
        ));
    }

    Ok(sum)
}

#[cfg(test)]
mod tests {
    use crate::{
        consts::{ARITHM_OF, BPOW_PRECISION},
        fixed::{bdiv, bmul, bpow, bpow_approx},
    };
    use frame_support::dispatch::DispatchError;
    use zeitgeist_primitives::constants::BASE;

    pub const ERR: Result<u128, DispatchError> = Err(ARITHM_OF);

    macro_rules! create_tests {
        (
            $op:ident;

            0 => $_0_0:expr, $_0_1:expr, $_0_2:expr, $_0_3:expr;
            1 => $_1_0:expr, $_1_1:expr, $_1_2:expr, $_1_3:expr;
            2 => $_2_0:expr, $_2_1:expr, $_2_2:expr, $_2_3:expr;
            3 => $_3_0:expr, $_3_1:expr, $_3_2:expr, $_3_3:expr;
            max_n => $max_n_0:expr, $max_n_1:expr, $max_n_2:expr, $max_n_3:expr;
            n_max => $n_max_0:expr, $n_max_1:expr, $n_max_2:expr, $n_max_3:expr;
        ) => {
            assert_eq!($op(0, 0 * BASE), $_0_0);
            assert_eq!($op(0, 1 * BASE), $_0_1);
            assert_eq!($op(0, 2 * BASE), $_0_2);
            assert_eq!($op(0, 3 * BASE), $_0_3);

            assert_eq!($op(1 * BASE, 0 * BASE), $_1_0);
            assert_eq!($op(1 * BASE, 1 * BASE), $_1_1);
            assert_eq!($op(1 * BASE, 2 * BASE), $_1_2);
            assert_eq!($op(1 * BASE, 3 * BASE), $_1_3);

            assert_eq!($op(2 * BASE, 0 * BASE), $_2_0);
            assert_eq!($op(2 * BASE, 1 * BASE), $_2_1);
            assert_eq!($op(2 * BASE, 2 * BASE), $_2_2);
            assert_eq!($op(2 * BASE, 3 * BASE), $_2_3);

            assert_eq!($op(3 * BASE, 0 * BASE), $_3_0);
            assert_eq!($op(3 * BASE, 1 * BASE), $_3_1);
            assert_eq!($op(3 * BASE, 2 * BASE), $_3_2);
            assert_eq!($op(3 * BASE, 3 * BASE), $_3_3);

            assert_eq!($op(u128::MAX, 0 * BASE), $max_n_0);
            assert_eq!($op(u128::MAX, 1 * BASE), $max_n_1);
            assert_eq!($op(u128::MAX, 2 * BASE), $max_n_2);
            assert_eq!($op(u128::MAX, 3 * BASE), $max_n_3);

            assert_eq!($op(0, u128::MAX), $n_max_0);
            assert_eq!($op(1, u128::MAX), $n_max_1);
            assert_eq!($op(2, u128::MAX), $n_max_2);
            assert_eq!($op(3, u128::MAX), $n_max_3);
        };
    }

    #[test]
    fn bdiv_has_minimum_set_of_correct_values() {
        create_tests!(
            bdiv;
            0 => ERR, Ok(0), Ok(0), Ok(0);
            1 => ERR, Ok(BASE), Ok(BASE / 2), Ok(BASE / 3);
            2 => ERR, Ok(2 * BASE), Ok(BASE), Ok(6666666667);
            3 => ERR, Ok(3 * BASE), Ok(3 * BASE / 2), Ok(BASE);
            max_n => ERR, ERR, ERR, ERR;
            n_max => Ok(0), Ok(1 / BASE), Ok(2 / BASE), Ok(3 / BASE);
        );
    }

    #[test]
    fn bmul_has_minimum_set_of_correct_values() {
        create_tests!(
            bmul;
            0 => Ok(0), Ok(0), Ok(0), Ok(0);
            1 => Ok(0), Ok(BASE), Ok(2 * BASE), Ok(3 * BASE);
            2 => Ok(0), Ok(2 * BASE), Ok(4 * BASE), Ok(6 * BASE);
            3 => Ok(0), Ok(3 * BASE), Ok(6 * BASE), Ok(9 * BASE);
            max_n => Ok(0), ERR, ERR, ERR;
            n_max => Ok(0), ERR, ERR, ERR;
        );
    }

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
        let test_vector: Vec<(u128, u128)> = vec![
            (BASE / 10, 3 * BASE / 2),
            (2 * BASE - BASE / 10, 3 * BASE / 2),
            (BASE, 11 * BASE / 10),
        ];
        for (base, exp) in test_vector.iter() {
            assert!(bpow_approx(*base, *exp).is_err());
        }
    }
}
