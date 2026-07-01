//! Operations on a Galois Field of 256 elements with the irreducible polynomial x^8 + x^4 + x^3 + x^2 + 1

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

#[allow(clippy::needless_range_loop)]
pub fn remainder(dividend: &[u8], divisor: &[u8]) -> Vec<u8> {
    let mut remainder = vec![0; divisor.len() - 1];

    for i in 0..dividend.len() {
        // At every step we cancel out the leading term in the remainder by choosing a factor such
        // that `remainder[0] + dividend[i] + divisor[0] * factor = 0`.
        // This is just `inv(divisor[0]) * (remainder[0] + dividend[i])`.
        let factor = add(dividend[i], remainder[0]);

        // I am too lazy to calculate the inverse, and since it's never actually needed we just assert `divisor[0] = 1`
        assert_eq!(divisor[0], 1);

        // We shift the remainder left. This is equivalent to multiplying the polynomial by x.
        remainder.rotate_left(1);
        *remainder.last_mut().unwrap() = 0;

        for j in 0..remainder.len() {
            remainder[j] = add(remainder[j], mul(factor, divisor[j + 1]));
        }
    }

    remainder
}

/// Constructs a generator polynomial using 2 as the primitive element.
///
/// The parameter `N` is the number of terms in the polynomial, i.e. its degree + 1
pub const fn generator_polynomial<const N: usize>() -> [u8; N] {
    let mut coefficients = [0; N];
    coefficients[N - 1] = 1;

    let mut i = 0;
    while i < N - 1 {
        multiply_by_binomial(&mut coefficients, pow(2, i));
        i += 1;
    }

    coefficients
}

const fn multiply_by_binomial(coefficients: &mut [u8], constant: u8) {
    // Set every coefficient except the last to the coefficient to the right of it + its value multiplied by the constant.
    // This is equivalent to multiplying the next coefficient by x, and the current coefficient by the constant.
    let mut i = 0;
    while i < coefficients.len() - 1 {
        coefficients[i] = add(coefficients[i + 1], mul(coefficients[i], constant));
        i += 1;
    }

    // Set the last coefficient to its value multiplied by the constant.
    coefficients[i] = mul(coefficients[i], constant);
}

/// Gets a generator polynomial of a specific degree from a precomputed set.
pub fn get_generator_polynomial(r: usize) -> &'static [u8] {
    match r {
        16 => const { &generator_polynomial::<{ 16 + 1 }>() },
        17 => const { &generator_polynomial::<{ 17 + 1 }>() },
        18 => const { &generator_polynomial::<{ 18 + 1 }>() },
        20 => const { &generator_polynomial::<{ 20 + 1 }>() },
        22 => const { &generator_polynomial::<{ 22 + 1 }>() },
        24 => const { &generator_polynomial::<{ 24 + 1 }>() },
        26 => const { &generator_polynomial::<{ 26 + 1 }>() },
        28 => const { &generator_polynomial::<{ 28 + 1 }>() },
        30 => const { &generator_polynomial::<{ 30 + 1 }>() },
        32 => const { &generator_polynomial::<{ 32 + 1 }>() },
        34 => const { &generator_polynomial::<{ 34 + 1 }>() },
        36 => const { &generator_polynomial::<{ 36 + 1 }>() },
        40 => const { &generator_polynomial::<{ 40 + 1 }>() },
        42 => const { &generator_polynomial::<{ 42 + 1 }>() },
        44 => const { &generator_polynomial::<{ 44 + 1 }>() },
        46 => const { &generator_polynomial::<{ 46 + 1 }>() },
        48 => const { &generator_polynomial::<{ 48 + 1 }>() },
        50 => const { &generator_polynomial::<{ 50 + 1 }>() },
        52 => const { &generator_polynomial::<{ 52 + 1 }>() },
        54 => const { &generator_polynomial::<{ 54 + 1 }>() },
        56 => const { &generator_polynomial::<{ 56 + 1 }>() },
        58 => const { &generator_polynomial::<{ 58 + 1 }>() },
        60 => const { &generator_polynomial::<{ 60 + 1 }>() },
        62 => const { &generator_polynomial::<{ 62 + 1 }>() },
        64 => const { &generator_polynomial::<{ 64 + 1 }>() },
        66 => const { &generator_polynomial::<{ 66 + 1 }>() },
        68 => const { &generator_polynomial::<{ 68 + 1 }>() },
        _ => panic!("invalid degree"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_is_xor() {
        assert_eq!(add(0, 0), 0);
        assert_eq!(add(0x12, 0x34), 0x26);
        assert_eq!(add(0xff, 0xff), 0);
    }

    #[test]
    fn add_identity() {
        for a in 0u8..=255 {
            assert_eq!(add(a, 0), a);
            assert_eq!(add(0, a), a);
        }
    }

    #[test]
    fn add_is_its_own_inverse() {
        for a in 0u8..=255 {
            assert_eq!(add(a, a), 0);
        }
    }

    #[test]
    fn multiplication_identity() {
        for a in 0u8..=255 {
            assert_eq!(mul(a, 1), a);
            assert_eq!(mul(1, a), a);
        }
    }

    #[test]
    fn multiplication_by_zero() {
        for a in 0u8..=255 {
            assert_eq!(mul(a, 0), 0);
            assert_eq!(mul(0, a), 0);
        }
    }

    #[test]
    fn multiplication_is_commutative() {
        for a in 0u8..=255 {
            for b in 0u8..=255 {
                assert_eq!(mul(a, b), mul(b, a));
            }
        }
    }

    #[test]
    fn multiplication_is_distributive() {
        for a in 0u8..32 {
            for b in 0u8..32 {
                for c in 0u8..32 {
                    assert_eq!(mul(a, add(b, c)), add(mul(a, b), mul(a, c)));
                }
            }
        }
    }

    #[test]
    fn powers() {
        for a in 0u8..=255 {
            assert_eq!(pow(a, 0), 1);
            assert_eq!(pow(a, 1), a);
            assert_eq!(pow(a, 2), mul(a, a));
            assert_eq!(pow(a, 3), mul(mul(a, a), a));
        }
    }

    #[test]
    fn nonzero_elements_have_order_255() {
        for a in 1u8..=255 {
            assert_eq!(pow(a, 255), 1);
        }
    }

    #[test]
    fn generator_polynomial_has_correct_shape() {
        let g = generator_polynomial::<17>();

        assert_eq!(g.len(), 17);
        assert_eq!(g[0], 1);
    }

    #[test]
    fn get_generator_polynomial_matches_const_version() {
        assert_eq!(get_generator_polynomial(16), &generator_polynomial::<17>());

        assert_eq!(get_generator_polynomial(30), &generator_polynomial::<31>());
    }

    #[test]
    fn remainder_of_zero_is_zero() {
        let divisor = generator_polynomial::<17>();

        let rem = remainder(&[0; 32], &divisor);

        assert_eq!(rem, vec![0; 16]);
    }

    #[test]
    fn remainder_has_correct_length() {
        let divisor = generator_polynomial::<17>();

        let rem = remainder(&[1, 2, 3, 4, 5], &divisor);

        assert_eq!(rem.len(), divisor.len() - 1);
    }

    #[test]
    fn remainder_of_zero_polynomial_is_zero() {
        let g = generator_polynomial::<17>();

        let rem = remainder(&[0; 32], &g);

        assert_eq!(rem, vec![0; 16]);
    }

    #[test]
    fn remainder_has_expected_degree() {
        let g = generator_polynomial::<17>();

        let rem = remainder(&[1, 2, 3, 4, 5], &g);

        assert_eq!(rem.len(), g.len() - 1);
    }

    #[test]
    fn codeword_has_zero_remainder() {
        let g = generator_polynomial::<17>();

        let message = [1, 2, 3, 4, 5, 6, 7, 8];

        let rem = remainder(&message, &g);

        let mut codeword = message.to_vec();
        codeword.extend_from_slice(&rem);

        assert_eq!(remainder(&codeword, &g), vec![0; g.len() - 1]);
    }

    #[test]
    fn empty_message_has_zero_remainder() {
        let g = generator_polynomial::<17>();

        assert_eq!(remainder(&[], &g), vec![0; 16]);
    }

    #[test]
    fn different_messages_produce_different_remainders() {
        let g = generator_polynomial::<17>();

        let r1 = remainder(&[1, 2, 3, 4], &g);
        let r2 = remainder(&[1, 2, 3, 5], &g);

        assert_ne!(r1, r2);
    }
}
