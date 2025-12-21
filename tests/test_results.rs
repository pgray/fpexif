//! Test result infrastructure for capturing JSON reports instead of panicking
//!
//! This module provides types and utilities for tests to capture their results
//! as JSON files, which can then be processed by CI to generate PR comments.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Directory where test results are written
pub const RESULTS_DIR: &str = "test-results";

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueCategory {
    UnknownTag,
    MissingField,
    ValueMismatch,
    TypeMismatch,
    CountMismatch,
    ExtraField,
    ParseError,
    Critical,
}

/// Results for a single file test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTestResult {
    pub file_path: String,
    pub format: String,
    pub success: bool,
    pub fpexif_tag_count: usize,
    pub reference_tag_count: usize,
    /// Number of tags that matched exactly between fpexif and reference tool
    #[serde(default)]
    pub matching_tags: usize,
    /// Number of tags with value mismatches
    #[serde(default)]
    pub mismatched_tags: usize,
    /// Number of tags missing from fpexif output
    #[serde(default)]
    pub missing_tags: usize,
    /// Number of extra tags in fpexif not in reference
    #[serde(default)]
    pub extra_tags: usize,
    pub issues: Vec<TestIssue>,
}

/// Aggregated results for a format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatTestResult {
    pub format: String,
    pub test_name: String,
    pub reference_tool: String,
    pub files_tested: usize,
    pub files_passed: usize,
    pub total_issues: usize,
    pub unknown_tags: usize,
    pub missing_fields: usize,
    pub value_mismatches: usize,
    pub critical_issues: usize,
    /// Total matching tags across all files
    #[serde(default)]
    pub total_matching_tags: usize,
    /// Total mismatched tags across all files
    #[serde(default)]
    pub total_mismatched_tags: usize,
    /// Total missing tags across all files
    #[serde(default)]
    pub total_missing_tags: usize,
    /// Total extra tags across all files
    #[serde(default)]
    pub total_extra_tags: usize,
    pub file_results: Vec<FileTestResult>,
}

impl FormatTestResult {
    pub fn new(format: &str, test_name: &str, reference_tool: &str) -> Self {
        Self {
            format: format.to_uppercase(),
            test_name: test_name.to_string(),
            reference_tool: reference_tool.to_string(),
            files_tested: 0,
            files_passed: 0,
            total_issues: 0,
            unknown_tags: 0,
            missing_fields: 0,
            value_mismatches: 0,
            critical_issues: 0,
            total_matching_tags: 0,
            total_mismatched_tags: 0,
            total_missing_tags: 0,
            total_extra_tags: 0,
            file_results: Vec::new(),
        }
    }

    pub fn add_file_result(&mut self, result: FileTestResult) {
        self.files_tested += 1;
        if result.success {
            self.files_passed += 1;
        }

        // Aggregate per-file tag counts
        self.total_matching_tags += result.matching_tags;
        self.total_mismatched_tags += result.mismatched_tags;
        self.total_missing_tags += result.missing_tags;
        self.total_extra_tags += result.extra_tags;

        for issue in &result.issues {
            self.total_issues += 1;
            match issue.category {
                IssueCategory::UnknownTag => self.unknown_tags += 1,
                IssueCategory::MissingField => self.missing_fields += 1,
                IssueCategory::ValueMismatch
                | IssueCategory::TypeMismatch
                | IssueCategory::CountMismatch => self.value_mismatches += 1,
                IssueCategory::Critical => self.critical_issues += 1,
                _ => {}
            }
        }

        self.file_results.push(result);
    }

    /// Write results to JSON file
    pub fn write_to_file(&self) -> std::io::Result<()> {
        let results_dir = Path::new(RESULTS_DIR);
        fs::create_dir_all(results_dir)?;

        let filename = format!(
            "{}-{}.json",
            self.test_name.replace("::", "-"),
            self.format.to_lowercase()
        );
        let path = results_dir.join(filename);

        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;

        Ok(())
    }

    /// Check if there are critical failures that should fail the test
    #[allow(dead_code)]
    pub fn has_critical_failures(&self) -> bool {
        self.critical_issues > 0
    }
}

/// Helper to create issue for missing field
#[allow(dead_code)]
pub fn missing_field_issue(field: &str) -> TestIssue {
    TestIssue {
        category: IssueCategory::MissingField,
        message: format!("Missing field in fpexif: {}", field),
        field: Some(field.to_string()),
        expected: None,
        actual: None,
    }
}

/// Helper to create issue for value mismatch
pub fn value_mismatch_issue(field: &str, expected: &str, actual: &str) -> TestIssue {
    TestIssue {
        category: IssueCategory::ValueMismatch,
        message: format!(
            "Value mismatch for {}: expected=\"{}\" actual=\"{}\"",
            field, expected, actual
        ),
        field: Some(field.to_string()),
        expected: Some(expected.to_string()),
        actual: Some(actual.to_string()),
    }
}

/// Determines if a missing field is critical (should fail the test)
#[allow(dead_code)]
pub fn is_critical_missing_field(field: &str) -> bool {
    // Brand-prefixed maker note fields are not critical
    let has_brand_prefix = field.starts_with("Canon")
        || field.starts_with("Nikon")
        || field.starts_with("Sony")
        || field.starts_with("Olympus")
        || field.starts_with("Panasonic")
        || field.starts_with("Pentax")
        || field.starts_with("Fuji")
        || field.starts_with("Kodak")
        || field.starts_with("Minolta")
        || field.starts_with("Exif.Canon")
        || field.starts_with("Exif.Nikon")
        || field.starts_with("Exif.Sony")
        || field.starts_with("Exif.Fuji")
        || field.starts_with("Exif.Olympus")
        || field.starts_with("Exif.Panasonic")
        || field.starts_with("Exif.Kodak")
        || field.starts_with("Exif.Minolta");

    // Derived/calculated fields that exiftool adds
    let is_derived = field == "Aperture"
        || field == "ShutterSpeed"
        || field == "ISO"
        || field == "LightValue"
        || field == "ImageSize"
        || field == "Megapixels"
        || field == "ScaleFactor35efl"
        || field == "FOV"
        || field == "HyperfocalDistance"
        || field == "CircleOfConfusion"
        || field == "FocalLength35efl";

    // File metadata
    let is_file_meta = field.starts_with("File")
        || field.starts_with("Directory")
        || field == "ExifByteOrder"
        || field == "ExifToolVersion"
        || field == "MIMEType";

    // Thumbnail/preview data
    let is_thumbnail = field.contains("Thumbnail")
        || field.contains("Preview")
        || field == "ThumbnailImage"
        || field == "PreviewImage"
        || field.starts_with("Exif.Thumbnail");

    // Interoperability IFD
    let is_interop = field.starts_with("Interop");

    // IPTC/XMP (not yet supported)
    let is_iptc_xmp = field.starts_with("Iptc.") || field.starts_with("Xmp.");

    // MakerNote raw data
    let is_makernote = field.contains("MakerNote");

    // Kodak-specific IFD fields (from KodakIFD sub-directory)
    let is_kodak_ifd = field == "KodakVersion"
        || field == "BatteryLevel"
        || field == "CFAPattern2"
        || field == "UnknownEV"
        || field.starts_with("KodakIFD");

    !(has_brand_prefix
        || is_derived
        || is_file_meta
        || is_thumbnail
        || is_interop
        || is_iptc_xmp
        || is_makernote
        || is_kodak_ifd)
}
