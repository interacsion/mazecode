use std::cmp::min;

use anyhow::{Result, bail};
use bitvec::{
    bits,
    field::BitField,
    order::{Lsb0, Msb0},
    vec::BitVec,
    view::{AsBits, BitView},
};

use crate::versions::{VERSION_GROUPS, Version, find_version_with_bit_capacity};

mod versions;

fn encode_data(data: &[u8]) -> Result<(Vec<u8>, Version)> {
    // TODO: mix different modes to optimize size

    for version_group in &VERSION_GROUPS {
        let data_len = data.len();
        let data_len_bits = data_len.view_bits::<Msb0>();

        if data_len_bits[..(data_len_bits.len() - version_group.byte_count_indicator_length)].any()
        {
            todo!()
        }

        let data_len_bits =
            &data_len_bits[(data_len_bits.len() - version_group.byte_count_indicator_length)..];
        debug_assert!(data_len_bits.len() == version_group.byte_count_indicator_length);
        debug_assert!(data_len_bits.load_be::<usize>() == data_len);

        let mut result: BitVec<u8, Msb0> = BitVec::new();
        result.extend_from_bitslice(bits![0, 1, 0, 0]); // byte mode indicator
        result.extend_from_bitslice(data_len_bits);
        result.extend_from_bitslice(data.as_bits::<Msb0>());

        let Some((version, bit_capacity)) =
            find_version_with_bit_capacity(result.len(), version_group.range)
        else {
            continue;
        };

        let terminator_len = min(4, bit_capacity - result.len());
        result.resize(result.len() + terminator_len, false);

        debug_assert!(result.len() % 8 == 0);

        loop {
            if result.len() == bit_capacity {
                break;
            }

            result.extend_from_bitslice(bits![1, 1, 1, 0, 1, 1, 0, 0]);

            if result.len() == bit_capacity {
                break;
            }

            result.extend_from_bitslice(bits![0, 0, 0, 1, 0, 0, 0, 1]);
        }

        return Ok((result.into_vec(), version));
    }

    bail!("input data is too long")
}

fn gf256_add(x: u8, y: u8) -> u8 {
    x ^ y
}

fn gf256_mul(x: u8, y: u8) -> u8 {
    todo!()
}

/// Calculates the Error Correction Codes (ECC) for given message data.
///
/// They are the remainder of dividing the message data interpreted as polynomial by some monic generator polynomial.
///
/// This is implemented via long division.
fn calculate_error_correction_codes(data: &[u8], count: usize, g: &[u8]) -> Vec<u8> {
    // The coefficients of the remainder
    let mut remainder = vec![0; count];

    for &byte in data {
        // We need a factor that cancels out the leading term.
        // Since we're in a Galois Field and g is monic, that factor is just the leading coefficient.
        let factor = gf256_add(byte, remainder[0]);

        // Shift all coefficients left. Equivalent to multiplying the remainder by x.
        for i in 0..(count - 1) {
            remainder[i] = remainder[i + 1];
        }
        remainder[count - 1] = 0;

        // Add the scaled polynomial to the remainder. This cancels out the leading term.
        for i in 0..count {
            remainder[i] = gf256_add(remainder[i], gf256_mul(factor, g[i + 1]))
        }
    }

    remainder
}

fn main() -> Result<()> {
    let (data, version) = encode_data(b"Hello, World!")?;

    Ok(())
}
