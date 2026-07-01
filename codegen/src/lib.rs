use std::{collections::BTreeMap, ops::RangeInclusive};

use quote::quote;
use serde::Deserialize;
use syn::{File, parse_quote};

#[derive(Deserialize, Debug)]
pub struct VersionInfo {
    groups: Vec<VersionGroup>,
    versions: BTreeMap<usize, Version>,
}

#[derive(Deserialize, Debug)]
struct VersionGroup {
    range: RangeInclusive<usize>,
    numeric_count_indicator_length: usize,
    alphanumeric_count_indicator_length: usize,
    byte_count_indicator_length: usize,
    kanji_count_indicator_length: usize,
}

#[derive(Deserialize, Debug)]
struct Version {
    data_codewords: usize,
    error_correction_blocks: Vec<ErrorCorrectionBlock>,
}

#[derive(Deserialize, Debug)]
struct ErrorCorrectionBlock {
    count: usize,
    total_codewords: usize,
    data_codewords: usize,
}

pub fn generate(version_info: &VersionInfo) -> File {
    let version_group = quote! {
        pub struct VersionGroup {
            pub range: std::range::RangeInclusive<super::Version>,
            pub numeric_count_indicator_length: usize,
            pub alphanumeric_count_indicator_length: usize,
            pub byte_count_indicator_length: usize,
            pub kanji_count_indicator_length: usize,
        }
    };

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

    let version_data_codewords = version_info.versions.iter().enumerate().map(|(index, (number, version))| {
        assert!(*number as usize == index + 1);
        version.data_codewords
    });

    parse_quote! {
        #version_group

        pub static VERSION_GROUPS: [VersionGroup; #version_groups_len] = [
            #(#version_groups),*
        ];

        pub static VERSION_DATA_CODEWORDS: [usize; #versions_len] = [
            #(#version_data_codewords),*
        ];
    }
}
