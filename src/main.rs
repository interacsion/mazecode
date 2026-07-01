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

fn encode_data(raw_data: &[u8]) -> Result<(Vec<u8>, Version)> {
    // TODO: mix different modes to optimize size

    for version_group in version_groups() {
        let data_len = raw_data.len();
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
        result.extend_from_bitslice(raw_data.as_bits::<Msb0>());

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

fn construct_message(raw_data: &[u8]) -> Result<(Vec<u8>, Version)> {
    let (data, version) = encode_data(raw_data)?;

    let blocks = get_version_error_correction_blocks(version);

    let mut data_blocks = Vec::with_capacity(blocks.len());
    let mut error_correction_blocks = Vec::with_capacity(blocks.len());
    let mut data_index = 0;

    for block in blocks {
        let data_block = &data[data_index..(data_index + block.data_codewords)];

        let error_correction_block = gf::remainder(
            data_block,
            gf::get_generator_polynomial(block.total_codewords - block.data_codewords),
        );

        data_blocks.push(data_block);
        error_correction_blocks.push(error_correction_block);
        
        data_index += block.data_codewords;
    }

    debug_assert_eq!(data_index, data.len());

    let mut result = Vec::new();

    let last_block = blocks.last().unwrap();

    for i in 0..last_block.data_codewords {
        for block in &data_blocks {
            result.extend(block.get(i));
        }
    }

    for i in 0..(last_block.total_codewords - last_block.data_codewords) {
        for block in &error_correction_blocks {
            result.extend(block.get(i));
        }
    }

    Ok((result, version))
}

fn main() -> Result<()> {
    let (data, version) = construct_message(b"Hello, World!")?;

    Ok(())
}
