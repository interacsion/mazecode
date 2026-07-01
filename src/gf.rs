//! This module contains operations on a Galois Field of 256 elements with
//! the irreducible polynomial x^8 + x^4 + x^3 + x^2 + 1

pub const fn add(a: u8, b: u8) -> u8 {
    a ^ b
}

pub const fn mul(mut a: u8, b: u8) -> u8 {
    let mut res = 0;

    let mut i = 0;
    while i < 8 {
        // If the current bit of `b` is set add `a` to the result
        if (b >> i) & 1 == 1 {
            res = add(res, a);
        }

        // Carry the leftmost bit of `a`
        let carry = a >> 7;

        // Shift `a` left. This is equivalent to multiplying the polynomial by x
        a <<= 1;

        // If the carried bit is set it is cancelled out by adding the irreducible polynomial
        if carry == 1 {
            a = add(a, 0b00011101);
        }

        i += 1;
    }

    res
}

pub const fn pow(b: u8, e: usize) -> u8 {
    let mut res = 1;

    let mut i = 0;
    while i < e {
        res = mul(res, b);
        i += 1;
    }

    res
}

/// Constructs a generator polynomial of a specified degree using 2 as the primitive element.
///
/// Note that for the function to return N coefficients, the leading coefficient is not returned and is implied to be 1.
const fn generator_polynomial<const N: usize>() -> [u8; N] {
    let mut coefficients = [0; N];
    coefficients[N - 1] = 1;

    let mut i = 0;
    while i < N {
        coefficients = multiply_by_binomial(coefficients, pow(2, i));
        i += 1;
    }

    coefficients
}

const fn multiply_by_binomial<const N: usize>(coefficients: [u8; N], constant: u8) -> [u8; N] {
    let mut res = [0; N];

    // Set every coefficient except the last to the coefficient to the right of it + its value multiplied by the constant.
    // This is equivalent to multiplying the next coefficient by x, and the current coefficient by the constant.
    let mut i = 0;
    while i < N - 1 {
        res[i] = add(coefficients[i + 1], mul(coefficients[i], constant));
        i += 1;
    }

    // Set the last coefficient to its value multiplied by the constant.
    res[N - 1] = mul(coefficients[N - 1], constant);

    res
}

fn get_generator_polynomial(n: usize) -> &'static [u8] {
    match n {
        16 => const { &generator_polynomial::<16>() },
        17 => const { &generator_polynomial::<17>() },
        18 => const { &generator_polynomial::<18>() },
        20 => const { &generator_polynomial::<20>() },
        22 => const { &generator_polynomial::<22>() },
        24 => const { &generator_polynomial::<24>() },
        26 => const { &generator_polynomial::<26>() },
        28 => const { &generator_polynomial::<28>() },
        30 => const { &generator_polynomial::<30>() },
        32 => const { &generator_polynomial::<32>() },
        34 => const { &generator_polynomial::<34>() },
        36 => const { &generator_polynomial::<36>() },
        40 => const { &generator_polynomial::<40>() },
        42 => const { &generator_polynomial::<42>() },
        44 => const { &generator_polynomial::<44>() },
        46 => const { &generator_polynomial::<46>() },
        48 => const { &generator_polynomial::<48>() },
        50 => const { &generator_polynomial::<50>() },
        52 => const { &generator_polynomial::<52>() },
        54 => const { &generator_polynomial::<54>() },
        56 => const { &generator_polynomial::<56>() },
        58 => const { &generator_polynomial::<58>() },
        60 => const { &generator_polynomial::<60>() },
        62 => const { &generator_polynomial::<62>() },
        64 => const { &generator_polynomial::<64>() },
        66 => const { &generator_polynomial::<66>() },
        68 => const { &generator_polynomial::<68>() },
        _ => panic!("invalid degree"),
    }
}
