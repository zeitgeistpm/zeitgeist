use crate::{CheckArithmRslt, BASE};
use frame_support::dispatch::DispatchError;

pub struct BPow;

impl BPow {
    const BPOW_PRECISION: u128 = BASE;

    pub fn new(base: u128, exp: u128) -> Result<u128, DispatchError> {
        if base % Self::BPOW_PRECISION != 0 {
            let msg = "Base value is greater than the maximum allowed integer part";
            return Err(DispatchError::Other(msg));
        }

        let whole = Self::bfloor(exp)?;
        let remain = exp.check_sub_rslt(&whole)?;

        let whole_pow = Self::bpowi(base, Self::btoi(whole)?)?;

        if remain == 0 {
            return whole_pow.check_div_rslt(&BASE);
        }

        let partial_result = Self::bpow_approx(base, exp, Self::BPOW_PRECISION)?;
        let rslt = Self::bmul(whole_pow, partial_result)?;
        rslt.check_div_rslt(&BASE)
    }

    fn bdiv(a: u128, b: u128) -> Result<u128, DispatchError> {
        let c0 = a.check_mul_rslt(&BASE)?;
        let c1 = c0.check_add_rslt(&b.check_div_rslt(&2)?)?;
        c1.check_div_rslt(&b)
    }

    fn bfloor(a: u128) -> Result<u128, DispatchError> {
        Self::btoi(a)?.check_mul_rslt(&BASE)
    }

    fn bmul(a: u128, b: u128) -> Result<u128, DispatchError> {
        let c0 = a.check_mul_rslt(&b)?;
        let c1 = c0.check_add_rslt(&BASE.check_div_rslt(&2)?)?;
        c1.check_div_rslt(&BASE)
    }

    fn bpowi(a: u128, n: u128) -> Result<u128, DispatchError> {
        let mut z = if n % 2 != 0 { a } else { BASE };

        let mut b = a;
        let mut m = n.check_div_rslt(&2)?;

        while m != 0 {
            b = Self::bmul(b, b)?;

            if m % 2 != 0 {
                z = Self::bmul(z, b)?;
            }

            m = m.check_div_rslt(&2)?;
        }

        Ok(z)
    }

    fn bpow_approx(base: u128, exp: u128, precision: u128) -> Result<u128, DispatchError> {
        let a = exp;
        let (x, xneg) = Self::bsub_sign(base, BASE)?;
        let mut term = BASE;
        let mut sum = term;
        let mut negative = false;

        // term(k) = numer / denom
        //         = (product(a - i - 1, i=1-->k) * x^k) / (k!)
        // each iteration, multiply previous term by (a-(k-1)) * x / k
        // continue until term is less than precision
        let mut i: u128 = 1;
        while term >= precision {
            let big_k = i.check_mul_rslt(&BASE)?;
            let (c, cneg) = Self::bsub_sign(a, big_k.check_sub_rslt(&BASE)?)?;
            term = Self::bmul(term, Self::bmul(c, x)?)?;
            term = Self::bdiv(term, big_k)?;
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

    fn bsub_sign(a: u128, b: u128) -> Result<(u128, bool), DispatchError> {
        if a >= b {
            Ok((a.check_sub_rslt(&b)?, false))
        } else {
            Ok((b.check_sub_rslt(&a)?, true))
        }
    }

    fn btoi(a: u128) -> Result<u128, DispatchError> {
        a.check_div_rslt(&BASE)
    }
}

#[test]
fn bpow_has_minimum_set_of_correct_values() {
    assert_eq!(BPow::new(0 * BASE, 0 * BASE), Ok(1));
    assert_eq!(BPow::new(0 * BASE, 1 * BASE), Ok(0));
    assert_eq!(BPow::new(0 * BASE, 2 * BASE), Ok(0));
    assert_eq!(BPow::new(0 * BASE, 3 * BASE), Ok(0));

    assert_eq!(BPow::new(1 * BASE, 0 * BASE), Ok(1));
    assert_eq!(BPow::new(1 * BASE, 1 * BASE), Ok(1));
    assert_eq!(BPow::new(1 * BASE, 2 * BASE), Ok(1));
    assert_eq!(BPow::new(1 * BASE, 3 * BASE), Ok(1));

    assert_eq!(BPow::new(2 * BASE, 0 * BASE), Ok(1));
    assert_eq!(BPow::new(2 * BASE, 1 * BASE), Ok(2));
    assert_eq!(BPow::new(2 * BASE, 2 * BASE), Ok(4));
    assert_eq!(BPow::new(2 * BASE, 3 * BASE), Ok(8));

    assert_eq!(BPow::new(3 * BASE, 0 * BASE), Ok(1));
    assert_eq!(BPow::new(3 * BASE, 1 * BASE), Ok(3));
    assert_eq!(BPow::new(3 * BASE, 2 * BASE), Ok(9));
    assert_eq!(BPow::new(3 * BASE, 3 * BASE), Ok(27));

    assert!(BPow::new(u128::MAX, 0 * BASE).is_err());
    assert!(BPow::new(u128::MAX, 1 * BASE).is_err());
    assert!(BPow::new(u128::MAX, 2 * BASE).is_err());
    assert!(BPow::new(u128::MAX, 3 * BASE).is_err());

    assert!(BPow::new(0, u128::MAX).is_err());
    assert!(BPow::new(1, u128::MAX).is_err());
    assert!(BPow::new(2, u128::MAX).is_err());
    assert!(BPow::new(3, u128::MAX).is_err());
}
