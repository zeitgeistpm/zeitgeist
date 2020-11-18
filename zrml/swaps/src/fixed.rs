use crate::consts::BASE;

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

pub fn bpow(a: u128, n: u128) -> u128 {
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
