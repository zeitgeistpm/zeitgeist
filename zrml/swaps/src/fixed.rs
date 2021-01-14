use crate::consts::{BASE, BPOW_PRECISION};

pub fn btoi(a: u128) -> u128 {
    a / BASE
}

pub fn bfloor(a: u128) -> u128 {
    btoi(a) * BASE
}

pub fn bsub_sign(a: u128, b: u128) -> (u128, bool) {
    if a >= b {
        return (a - b, false);
    } else {
        return (b - a, true);
    }
}

pub fn bmul(a: u128, b: u128) -> u128 {
    let c0 = a * b;
    let c1 = c0 + BASE / 2;
    c1 / BASE
}

pub fn bdiv(a: u128, b: u128) -> u128 {
    let c0 = a * BASE;
    let c1 = c0 + b / 2;
    c1 / b
}

pub fn bpowi(a: u128, n: u128) -> u128 {
    let mut z = if n % 2 != 0 { a } else { BASE };

    let mut b = a;
    let mut m = n / 2;

    while m != 0 {
        b = bmul(b, b);

        if m % 2 != 0 {
            z = bmul(z, b);
        }

        m = m / 2;
    }

    z
}

pub fn bpow(base: u128, exp: u128) -> u128 {
    let whole = bfloor(exp);
    let remain = exp - whole;

    let whole_pow = bpowi(base, btoi(whole));

    if remain == 0 {
        return whole_pow;
    }

    let partial_result = bpow_approx(base, remain, BPOW_PRECISION);
    return bmul(whole_pow, partial_result);
}

pub fn bpow_approx(base: u128, exp: u128, precision: u128) -> u128 {
    let a = exp;
    let (x, xneg) = bsub_sign(base, BASE);
    let mut term = BASE;
    let mut sum = term;
    let mut negative = false;

    // term(k) = numer / denom
    //         = (product(a - i - 1, i=1-->k) * x^k) / (k!)
    // each iteration, multiply previous term by (a-(k-1)) * x / k
    // continue until term is less than precision
    let mut i = 1;
    while term >= precision {
        let big_k = i * BASE;
        let (c, cneg) = bsub_sign(a, big_k - BASE);
        term = bmul(term, bmul(c, x));
        term = bdiv(term, big_k);
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
            sum = sum - term;
        } else {
            sum = sum + term;
        }

        i += 1;
    }

    sum
}
