use std::cmp::min;

use anyhow::{Result, bail};
use bitvec::{
    bits,
    field::BitField,
    order::{Lsb0, Msb0},
    vec::BitVec,
    view::{AsBits, BitView},
};

use crate::versions::{
    Version, find_version_with_bit_capacity,
    get_version_error_correction_blocks, version_groups,
};

mod gf;
mod versions;

fn encode_data(data: &[u8]) -> Result<(Vec<u8>, Version)> {
    // TODO: mix different modes to optimize size

    for version_group in version_groups() {
        let data_len = data.len();
        let data_len_bits = data_len.view_bits::<Msb0>();

        if data_len_bits[..(data_len_bits.len() - version_group.byte_count_indicator_length)].any()
        {
            todo!()
        }

        let data_len_bits =
            &data_len_bits[(data_len_bits.len() - version_group.byte_count_indicator_length)..];
        debug_assert_eq!(data_len_bits.len(), version_group.byte_count_indicator_length);
        debug_assert_eq!(data_len_bits.load_be::<usize>(), data_len);

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

        debug_assert_eq!(result.len() % 8, 0);

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

fn main() -> Result<()> {
    let (data, version) = encode_data(b"Hello, World!")?;

    dbg!(version, &data);

    let mut data = data.into_iter();

    for block in get_version_error_correction_blocks(version) {
        let error_correction_block_count = block.total_codewords - block.data_codewords;

        todo!()
    }

    Ok(())
}
