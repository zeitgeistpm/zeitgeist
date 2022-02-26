use crate::{check_arithm_rslt::CheckArithmRslt, consts::BPOW_PRECISION};
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

pub fn bpow_approx(base: u128, exp: u128) -> Result<u128, DispatchError> {
    let a = exp;
    let (x, xneg) = bsub_sign(base, BASE)?;
    let mut term = BASE;
    let mut sum = term;
    let mut negative = false;

    // term(k) = numer / denom
    //         = (product(a - i - 1, i=1-->k) * x^k) / (k!)
    // each iteration, multiply previous term by (a-(k-1)) * x / k
    // continue until term is less than precision
    let mut i = 1;
    while term >= BPOW_PRECISION {
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
            sum = sum.check_sub_rslt(&term)?;
        } else {
            sum = sum.check_add_rslt(&term)?;
        }

        i = i.check_add_rslt(&1)?;
    }

    Ok(sum)
}

#[cfg(test)]
mod tests {
    use crate::{
        consts::ARITHM_OF,
        fixed::{bdiv, bmul, bpow},
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
        create_tests!(
            bpow;
            0 => Ok(BASE), Ok(0), Ok(0), Ok(0);
            1 => Ok(BASE), Ok(BASE), Ok(BASE), Ok(BASE);
            2 => Ok(BASE), Ok(2 * BASE), Ok(4 * BASE), Ok(8 * BASE);
            3 => Ok(BASE), Ok(3 * BASE), Ok(9 * BASE), Ok(27 * BASE);
            max_n => Ok(BASE), Ok(u128::MAX), ERR, ERR;
            n_max => ERR, ERR, ERR, ERR;
        );
    }
}
