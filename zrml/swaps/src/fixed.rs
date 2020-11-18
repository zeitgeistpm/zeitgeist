use crate::consts::BASE;

pub fn btoi(a: u128) -> u128 {
    a / BASE
}

pub fn bfloor(a: u128) -> u128 {
    btoi(a) * BASE
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

    let wholePow = bpowi(base, btoi(whole));

    if remain == 0 {
        return wholePow;
    }

    let partialResult = bpowApprox(base, remain, BPOW_PRECISION);
    return bmul(wholePow, partialResult);
}

pub fn bpowApprox(base: u128, exp: u128, precision: u128) -> u128 {

}
