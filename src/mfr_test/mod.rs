//! Manufacturer-specific EXIF testing module
//!
//! This module provides tools for testing EXIF tag implementations
//! for specific camera manufacturers. It supports:
//! - Saving baseline snapshots before starting work
//! - Comparing current state against baselines
//! - Comparing against exiftool output (ground truth)

pub mod baseline;
pub mod comparison;
pub mod output;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported manufacturers and their file formats
/// Note: "dng" is a format, not a manufacturer, but is included for testing DNG-specific parsing
pub const MANUFACTURER_FORMATS: &[(&str, &[&str])] = &[
    ("canon", &["CR2", "CR3", "CRW"]),
    ("nikon", &["NEF", "NRW"]),
    ("sony", &["ARW", "SR2", "SRF"]),
    ("fujifilm", &["RAF"]),
    ("panasonic", &["RW2"]),
    ("olympus", &["ORF"]),
    ("dng", &["DNG"]),
];

/// Get formats for a manufacturer (case-insensitive)
pub fn get_formats_for_manufacturer(manufacturer: &str) -> Option<&'static [&'static str]> {
    let mfr_lower = manufacturer.to_lowercase();
    MANUFACTURER_FORMATS
        .iter()
        .find(|(m, _)| *m == mfr_lower)
        .map(|(_, formats)| *formats)
}

/// Get all supported manufacturer names
pub fn get_supported_manufacturers() -> Vec<&'static str> {
    MANUFACTURER_FORMATS.iter().map(|(m, _)| *m).collect()
}

/// Issue category for test tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueCategory {
    UnknownTag,
    MissingField,
    ValueMismatch,
    TypeMismatch,
    ExtraField,
    ParseError,
    Critical,
}

/// A single issue found during testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestIssue {
    pub category: IssueCategory,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<String>,
}

/// Per-tag comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagComparison {
    pub tag_name: String,
    pub fpexif_value: Option<String>,
    pub exiftool_value: Option<String>,
    pub matches: bool,
}

/// Results for a single file test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTestResult {
    pub file_path: String,
    pub file_name: String,
    pub format: String,
    pub success: bool,
    pub fpexif_tag_count: usize,
    pub exiftool_tag_count: usize,
    pub matching_tags: usize,
    pub mismatched_tags: usize,
    pub missing_tags: usize,
    pub extra_tags: usize,
    #[serde(default)]
    pub tags: HashMap<String, TagComparison>,
    #[serde(default)]
    pub issues: Vec<TestIssue>,
}

/// Aggregated results for a manufacturer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufacturerTestResult {
    pub manufacturer: String,
    pub formats: Vec<String>,
    pub files_tested: usize,
    pub files_passed: usize,
    pub total_matching_tags: usize,
    pub total_mismatched_tags: usize,
    pub total_missing_tags: usize,
    pub total_extra_tags: usize,
    pub total_issues: usize,
    pub file_results: Vec<FileTestResult>,
}

impl ManufacturerTestResult {
    pub fn new(manufacturer: &str, formats: Vec<String>) -> Self {
        Self {
            manufacturer: manufacturer.to_string(),
            formats,
            files_tested: 0,
            files_passed: 0,
            total_matching_tags: 0,
            total_mismatched_tags: 0,
            total_missing_tags: 0,
            total_extra_tags: 0,
            total_issues: 0,
            file_results: Vec::new(),
        }
    }

    pub fn add_file_result(&mut self, result: FileTestResult) {
        self.files_tested += 1;
        if result.success {
            self.files_passed += 1;
        }
        self.total_matching_tags += result.matching_tags;
        self.total_mismatched_tags += result.mismatched_tags;
        self.total_missing_tags += result.missing_tags;
        self.total_extra_tags += result.extra_tags;
        self.total_issues += result.issues.len();
        self.file_results.push(result);
    }
}

/// Diff between baseline and current state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineDiff {
    pub baseline_commit: String,
    pub baseline_date: String,
    pub current_commit: String,

    // Summary deltas
    pub matching_delta: i64,
    pub mismatched_delta: i64,
    pub missing_delta: i64,
    pub extra_delta: i64,

    // Detailed changes
    pub improvements: Vec<TagChange>,
    pub regressions: Vec<TagChange>,
    pub new_files: Vec<String>,
    pub removed_files: Vec<String>,
}

/// A single tag change between baseline and current
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagChange {
    pub file: String,
    pub tag: String,
    pub was: String,
    pub now: String,
}
