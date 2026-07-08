use std::range::RangeInclusive;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

use generated::*;

#[derive(Copy, Clone, Debug)]
pub struct Version(usize);

impl Version {
    pub fn number(&self) -> u32 {
        self.0 as u32 + 1
    }

    pub fn size(&self) -> usize {
        21 + self.0 * 4
    }

    pub fn data_codewords(&self) -> usize {
        VERSION_DATA_CODEWORDS[self.0]
    }

    pub fn error_correction_blocks(&self) -> &'static [ErrorCorrectionBlock] {
        VERSION_ERROR_CORRECTION_BLOCKS[self.0]
    }

    pub fn alignment_pattern_coordinates(&self) -> &'static [usize] {
        VERSION_ALIGNMENT_PATTERN_COORDINATES[self.0]
    }
}

pub fn version_groups() -> &'static [VersionGroup] {
    &VERSION_GROUPS
}

pub fn find_version_with_data_capacity(
    capacity: usize,
    version_range: RangeInclusive<Version>,
) -> Option<Version> {
    VERSION_DATA_CODEWORDS
        .iter()
        .skip(version_range.start.0)
        .take(version_range.last.0 + 1)
        .position(|&codewords| codewords >= capacity)
        .map(Version)
}
