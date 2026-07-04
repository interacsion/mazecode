use std::{collections::BTreeMap, iter, ops::RangeInclusive};

use quote::quote;
use serde::Deserialize;
use syn::{File, parse_quote};

#[derive(Deserialize)]
pub struct VersionInfo {
    groups: Vec<VersionGroup>,
    versions: BTreeMap<usize, Version>,
}

#[derive(Deserialize)]
struct VersionGroup {
    range: RangeInclusive<usize>,
    numeric_count_indicator_length: usize,
    alphanumeric_count_indicator_length: usize,
    byte_count_indicator_length: usize,
    kanji_count_indicator_length: usize,
}

#[derive(Deserialize)]
struct Version {
    data_codewords: usize,
    error_correction_blocks: Vec<ErrorCorrectionBlock>,
    alignment_pattern_coordinates: Vec<usize>,
}

#[derive(Deserialize)]
struct ErrorCorrectionBlock {
    count: usize,
    total_codewords: usize,
    data_codewords: usize,
}

pub fn generate(version_info: &VersionInfo) -> File {
    let version_groups_len = version_info.groups.len();

    let version_groups = version_info.groups.iter().map(|group| {
        let VersionGroup {
            range,
            numeric_count_indicator_length,
            alphanumeric_count_indicator_length,
            byte_count_indicator_length,
            kanji_count_indicator_length,
        } = group;

        let range_start = range.start() - 1;
        let range_end = range.end() - 1;

        quote! {
            VersionGroup {
                range: std::range::RangeInclusive { start: super::Version(#range_start), last: super::Version(#range_end) },
                numeric_count_indicator_length: #numeric_count_indicator_length,
                alphanumeric_count_indicator_length: #alphanumeric_count_indicator_length,
                byte_count_indicator_length: #byte_count_indicator_length,
                kanji_count_indicator_length: #kanji_count_indicator_length,
            }
        }
    });

    let versions_len = version_info.versions.len();

    let version_data_codewords =
        version_info
            .versions
            .iter()
            .enumerate()
            .map(|(index, (number, version))| {
                assert!(*number == index + 1);
                version.data_codewords
            });

    let version_error_correction_blocks =
        version_info
            .versions
            .iter()
            .enumerate()
            .map(|(index, (number, version))| {
                assert!(*number == index + 1);

                let blocks = version.error_correction_blocks.iter().flat_map(|block| {
                    let ErrorCorrectionBlock {
                        count,
                        total_codewords,
                        data_codewords,
                    } = block;

                    iter::repeat_n(
                        quote! {
                            ErrorCorrectionBlock {
                                total_codewords: #total_codewords,
                                data_codewords: #data_codewords
                            }
                        },
                        *count,
                    )
                });

                quote! {
                    &[ #(#blocks),* ]
                }
            });

    let version_alignment_pattern_coordinates =
        version_info
            .versions
            .iter()
            .enumerate()
            .map(|(index, (number, version))| {
                assert!(*number == index + 1);

                let alignment_pattern_coordinates = &version.alignment_pattern_coordinates;

                quote! {
                    &[ #(#alignment_pattern_coordinates),* ]
                }
            });

    parse_quote! {
        #[derive(Debug)]
        pub struct VersionGroup {
            pub range: std::range::RangeInclusive<super::Version>,
            pub numeric_count_indicator_length: usize,
            pub alphanumeric_count_indicator_length: usize,
            pub byte_count_indicator_length: usize,
            pub kanji_count_indicator_length: usize,
        }

        pub static VERSION_GROUPS: [VersionGroup; #version_groups_len] = [
            #(#version_groups),*
        ];

        pub static VERSION_DATA_CODEWORDS: [usize; #versions_len] = [
            #(#version_data_codewords),*
        ];

        #[derive(Debug)]
        pub struct ErrorCorrectionBlock {
            pub total_codewords: usize,
            pub data_codewords: usize,
        }

        pub static VERSION_ERROR_CORRECTION_BLOCKS: [&[ErrorCorrectionBlock]; #versions_len] = [
            #(#version_error_correction_blocks),*
        ];

        pub static VERSION_ALIGNMENT_PATTERN_COORDINATES: [&[usize]; #versions_len] = [
            #(#version_alignment_pattern_coordinates),*
        ];
    }
}
