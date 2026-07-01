use std::range::RangeInclusive;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

pub use generated::VERSION_GROUPS;
use generated::*;

#[derive(Copy, Clone)]
pub struct Version(usize);

pub fn find_version_with_bit_capacity(
    bit_len: usize,
    version_range: RangeInclusive<Version>,
) -> Option<(Version, usize)> {
    dbg!(
        VERSION_DATA_CODEWORDS
            .iter()
            .skip(version_range.start.0)
            .take(version_range.last.0 + 1)
    )
    .enumerate()
    .find_map(|(index, codewords)| {
        (codewords * 8 >= bit_len).then_some((Version(index), codewords * 8))
    })
}
