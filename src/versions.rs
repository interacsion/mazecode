use std::range::RangeInclusive;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

use generated::*;

#[derive(Copy, Clone, Debug)]
pub struct Version(usize);

pub fn version_groups() -> &'static [VersionGroup] {
    &VERSION_GROUPS
}

pub fn find_version_with_bit_capacity(
    bit_len: usize,
    version_range: RangeInclusive<Version>,
) -> Option<(Version, usize)> {
    VERSION_DATA_CODEWORDS
        .iter()
        .skip(version_range.start.0)
        .take(version_range.last.0 + 1)
        .enumerate()
        .find_map(|(index, codewords)| {
            (codewords * 8 >= bit_len).then_some((Version(index), codewords * 8))
        })
}

pub fn get_version_error_correction_blocks(version: Version) -> &'static [ErrorCorrectionBlock] {
    VERSION_ERROR_CORRECTION_BLOCKS[version.0]
}
