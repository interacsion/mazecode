use std::{cmp::min, debug_assert_matches, fs};

use anyhow::{Result, bail};
use bitvec::{
    bits, bitvec,
    field::BitField,
    order::{Lsb0, Msb0},
    vec::BitVec,
    view::{AsBits, BitView},
};
use image::{ImageBuffer, ImageFormat, Rgba};

use crate::versions::{Version, find_version_with_bit_capacity, version_groups};

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
        debug_assert_eq!(
            data_len_bits.len(),
            version_group.byte_count_indicator_length
        );
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

    let blocks = version.error_correction_blocks();

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
    let (data, version) = construct_message(b"https://discord.gg/vestibule")?;

    let mut grid = bitvec![0; version.size() * version.size()];

    finder_pattern(&mut grid, version.size(), 0, 0);
    finder_pattern(&mut grid, version.size(), version.size() - 7, 0);
    finder_pattern(&mut grid, version.size(), 0, version.size() - 7);

    for i in (8..(version.size() - 8)) {
        grid.set(i + version.size() * 6, i % 2 == 0);
        grid.set(6 + version.size() * i, i % 2 == 0);
    }

    for &x in version.alignment_pattern_coordinates() {
        for &y in version.alignment_pattern_coordinates() {
            if x < 9 && y < 9 {
                continue;
            }

            if x < 9 && y > version.size() - 9 {
                continue;
            }

            if x > version.size() - 9 && y < 9 {
                continue;
            }

            alignment_pattern(&mut grid, version.size(), x, y);
        }
    }

    let mut data = data
        .iter()
        .flat_map(|codeword| codeword.view_bits::<Msb0>());

    for (x, y) in data_layout(version) {
        if let Some(bit) = data.next() {
            // data mask 001 for now
            let mask = y % 2 == 0;

            grid.set(x + y * version.size(), *bit ^ mask);
        }
    }

    debug_assert_matches!(data.next(), None);

    // BCH encoded format information
    let format_information = bitvec::bitvec![1, 0, 0, 0, 1, 1, 1, 1, 0, 1, 0, 1, 1, 0, 0];
    let masked_format_information = bitvec::bitarr![0, 0, 1, 0, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0];

    #[allow(clippy::erasing_op)]
    grid.set(8 + 0 * version.size(), masked_format_information[14]);
    #[allow(clippy::identity_op)]
    grid.set(8 + 1 * version.size(), masked_format_information[13]);
    grid.set(8 + 2 * version.size(), masked_format_information[12]);
    grid.set(8 + 3 * version.size(), masked_format_information[11]);
    grid.set(8 + 4 * version.size(), masked_format_information[10]);
    grid.set(8 + 5 * version.size(), masked_format_information[9]);
    grid.set(8 + 7 * version.size(), masked_format_information[8]);
    grid.set(8 + 8 * version.size(), masked_format_information[7]);
    grid.set(7 + 8 * version.size(), masked_format_information[6]);
    grid.set(5 + 8 * version.size(), masked_format_information[5]);
    grid.set(4 + 8 * version.size(), masked_format_information[4]);
    grid.set(3 + 8 * version.size(), masked_format_information[3]);
    grid.set(2 + 8 * version.size(), masked_format_information[2]);
    grid.set(1 + 8 * version.size(), masked_format_information[1]);
    #[allow(clippy::identity_op)]
    grid.set(0 + 8 * version.size(), masked_format_information[0]);

    grid.set(
        version.size() - 1 + 8 * version.size(),
        masked_format_information[14],
    );
    grid.set(
        version.size() - 2 + 8 * version.size(),
        masked_format_information[13],
    );
    grid.set(
        version.size() - 3 + 8 * version.size(),
        masked_format_information[12],
    );
    grid.set(
        version.size() - 4 + 8 * version.size(),
        masked_format_information[11],
    );
    grid.set(
        version.size() - 5 + 8 * version.size(),
        masked_format_information[10],
    );
    grid.set(
        version.size() - 6 + 8 * version.size(),
        masked_format_information[9],
    );
    grid.set(
        version.size() - 7 + 8 * version.size(),
        masked_format_information[8],
    );
    grid.set(
        version.size() - 8 + 8 * version.size(),
        masked_format_information[7],
    );

    // dark module
    grid.set(8 + (version.size() - 8) * version.size(), true);
    grid.set(
        8 + (version.size() - 7) * version.size(),
        masked_format_information[6],
    );
    grid.set(
        8 + (version.size() - 6) * version.size(),
        masked_format_information[5],
    );
    grid.set(
        8 + (version.size() - 5) * version.size(),
        masked_format_information[4],
    );
    grid.set(
        8 + (version.size() - 4) * version.size(),
        masked_format_information[3],
    );
    grid.set(
        8 + (version.size() - 3) * version.size(),
        masked_format_information[2],
    );
    grid.set(
        8 + (version.size() - 2) * version.size(),
        masked_format_information[1],
    );
    grid.set(
        8 + (version.size() - 1) * version.size(),
        masked_format_information[0],
    );

    display(&grid, version.size())?;

    Ok(())
}

fn data_layout(version: Version) -> Vec<(usize, usize)> {
    let mut result = Vec::new();

    enum Direction {
        Up,
        Down,
    }

    let mut direction = Direction::Up;
    let mut x = version.size() - 1;
    let mut y = version.size() - 1;

    loop {
        if x == 6 {
            x -= 1;
        }

        if can_place(x, y, version) {
            result.push((x, y));
        }

        if can_place(x - 1, y, version) {
            result.push((x - 1, y));
        }

        match direction {
            Direction::Up => {
                if y == 0 && x == 1 {
                    break;
                }

                if y == 0 {
                    direction = Direction::Down;
                    x -= 2;
                    continue;
                }

                y -= 1;
            }
            Direction::Down => {
                if y == version.size() - 1 && x == 1 {
                    break;
                }

                if y == version.size() - 1 {
                    direction = Direction::Up;
                    x -= 2;
                    continue;
                }

                y += 1;
            }
        }
    }

    result
}

fn can_place(x: usize, y: usize, version: Version) -> bool {
    // Top-left finder pattern area.
    if x <= 8 && y <= 8 {
        return false;
    }

    // Top-right finder pattern area.
    if x >= version.size() - 8 && y <= 8 {
        return false;
    }

    // Bottom-left finder pattern area.
    if x <= 8 && y >= version.size() - 8 {
        return false;
    }

    // Timing patterns run along row 6 and column 6.
    if x == 6 || y == 6 {
        return false;
    }

    // The dark module is a fixed reserved module.
    if x == 8 && y == (4 * version.number() as usize + 9) {
        return false;
    }

    // Format information appears around the finder patterns.
    if (x <= 8 && y == 8)
        || (y <= 8 && x == 8)
        || (x >= version.size() - 8 && y == 8)
        || (y >= version.size() - 8 && x == 8)
    {
        return false;
    }

    // Version information exists only for version 7 and above.
    // It occupies two 6x3 regions near the top-right and bottom-left.
    if version.number() >= 7
        && ((x >= version.size() - 11 && y < 6) || (y >= version.size() - 11 && x < 6))
    {
        return false;
    }

    // Alignment patterns are 5x5 blocks centered at every combination
    // of the provided alignment coordinates.
    for &cx in version.alignment_pattern_coordinates() {
        for &cy in version.alignment_pattern_coordinates() {
            if (cx <= 8 && cy <= 8)
                || (cx >= version.size() - 8 && cy <= 8)
                || (cx <= 8 && cy >= version.size() - 8)
            {
                continue;
            }

            if x.abs_diff(cx) <= 2 && y.abs_diff(cy) <= 2 {
                return false;
            }
        }
    }

    true
}

#[allow(clippy::identity_op)]
fn finder_pattern(grid: &mut BitVec, size: usize, x: usize, y: usize) {
    grid.set(x + 0 + size * y, true);
    grid.set(x + 1 + size * y, true);
    grid.set(x + 2 + size * y, true);
    grid.set(x + 3 + size * y, true);
    grid.set(x + 4 + size * y, true);
    grid.set(x + 5 + size * y, true);
    grid.set(x + 6 + size * y, true);

    grid.set(x + 0 + size * (y + 1), true);
    grid.set(x + 6 + size * (y + 1), true);

    grid.set(x + 0 + size * (y + 2), true);
    grid.set(x + 2 + size * (y + 2), true);
    grid.set(x + 3 + size * (y + 2), true);
    grid.set(x + 4 + size * (y + 2), true);
    grid.set(x + 6 + size * (y + 2), true);

    grid.set(x + 0 + size * (y + 3), true);
    grid.set(x + 2 + size * (y + 3), true);
    grid.set(x + 3 + size * (y + 3), true);
    grid.set(x + 4 + size * (y + 3), true);
    grid.set(x + 6 + size * (y + 3), true);

    grid.set(x + 0 + size * (y + 4), true);
    grid.set(x + 2 + size * (y + 4), true);
    grid.set(x + 3 + size * (y + 4), true);
    grid.set(x + 4 + size * (y + 4), true);
    grid.set(x + 6 + size * (y + 4), true);

    grid.set(x + 0 + size * (y + 5), true);
    grid.set(x + 6 + size * (y + 5), true);

    grid.set(x + 0 + size * (y + 6), true);
    grid.set(x + 1 + size * (y + 6), true);
    grid.set(x + 2 + size * (y + 6), true);
    grid.set(x + 3 + size * (y + 6), true);
    grid.set(x + 4 + size * (y + 6), true);
    grid.set(x + 5 + size * (y + 6), true);
    grid.set(x + 6 + size * (y + 6), true);
}

#[allow(clippy::identity_op)]
fn alignment_pattern(grid: &mut BitVec, size: usize, x: usize, y: usize) {
    grid.set(x - 2 + size * (y - 2), true);
    grid.set(x - 1 + size * (y - 2), true);
    grid.set(x + 0 + size * (y - 2), true);
    grid.set(x + 1 + size * (y - 2), true);
    grid.set(x + 2 + size * (y - 2), true);

    grid.set(x - 2 + size * (y - 1), true);
    grid.set(x - 1 + size * (y - 1), false);
    grid.set(x + 0 + size * (y - 1), false);
    grid.set(x + 1 + size * (y - 1), false);
    grid.set(x + 2 + size * (y - 1), true);

    grid.set(x - 2 + size * y, true);
    grid.set(x - 1 + size * y, false);
    grid.set(x + 0 + size * y, true);
    grid.set(x + 1 + size * y, false);
    grid.set(x + 2 + size * y, true);

    grid.set(x - 2 + size * (y + 1), true);
    grid.set(x - 1 + size * (y + 1), false);
    grid.set(x + 0 + size * (y + 1), false);
    grid.set(x + 1 + size * (y + 1), false);
    grid.set(x + 2 + size * (y + 1), true);

    grid.set(x - 2 + size * (y + 2), true);
    grid.set(x - 1 + size * (y + 2), true);
    grid.set(x + 0 + size * (y + 2), true);
    grid.set(x + 1 + size * (y + 2), true);
    grid.set(x + 2 + size * (y + 2), true);
}

/// terrible display implementation
fn display(grid: &BitVec, size: usize) -> Result<()> {
    let scale = 16;

    let mut imgbuf: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::new(scale * (size as u32 + 8), scale * (size as u32 + 8));

    imgbuf.fill(255);

    for x in 0..size {
        for y in 0..size {
            for x1 in 0..scale {
                for y1 in 0..scale {
                    if grid[x + y * size] {
                        imgbuf.put_pixel(
                            scale * (x as u32 + 4) + x1,
                            scale * (y as u32 + 4) + y1,
                            Rgba([0, 0, 0, 255]),
                        );
                    }
                }
            }
        }
    }

    imgbuf.write_to(&mut fs::File::create("out.png")?, ImageFormat::Png)?;

    Ok(())
}
