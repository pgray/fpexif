//! Comparison logic for exiftool and baseline comparisons

use super::{
    baseline::{get_git_commit, Baseline},
    BaselineDiff, FileTestResult, IssueCategory, ManufacturerTestResult, TagChange, TagComparison,
    TestIssue,
};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Get the test files directory from environment or default
pub fn get_test_files_dir() -> String {
    std::env::var("FPEXIF_TEST_FILES").unwrap_or_else(|_| "/fpexif/raws".to_string())
}

/// Check if the test files directory exists
pub fn test_files_exist() -> bool {
    Path::new(&get_test_files_dir()).exists()
}

/// Check if exiftool is available
pub fn exiftool_available() -> bool {
    Command::new("exiftool").arg("--version").output().is_ok()
}

/// Get exiftool JSON output for a file
fn get_exiftool_json(path: &str) -> Result<serde_json::Value, String> {
    let output = Command::new("exiftool")
        .arg("-json")
        .arg(path)
        .output()
        .map_err(|e| format!("Failed to run exiftool: {}", e))?;

    serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse exiftool JSON: {}", e))
}

/// Get fpexif --exiftool-json output for a file
fn get_fpexif_json(path: &str) -> Result<serde_json::Value, String> {
    let output = Command::new("cargo")
        .args(["run", "--features", "cli", "--bin", "fpexif", "--"])
        .args(["list", path, "--exiftool-json"])
        .output()
        .map_err(|e| format!("Failed to run fpexif: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "fpexif failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse fpexif JSON: {}", e))
}

/// Fields to ignore when comparing (exiftool-specific or expected to differ)
const IGNORE_FIELDS: &[&str] = &[
    // File metadata (not EXIF)
    "SourceFile",
    "ExifToolVersion",
    "FileName",
    "Directory",
    "FileSize",
    "FileModifyDate",
    "FileAccessDate",
    "FileInodeChangeDate",
    "FilePermissions",
    "FileType",
    "FileTypeExtension",
    "MIMEType",
    "CurrentIPTCDigest",
    "CodedCharacterSet",
    "ApplicationRecordVersion",
    "XMPToolkit",
    "MakerNote",
    // Binary preview/thumbnail data (we don't extract these)
    "PreviewImage",
    "PreviewImageStart",
    "PreviewImageLength",
    "ThumbnailImage",
    "ThumbnailTIFF",
    "ThumbnailOffset",
    "ThumbnailLength",
    "JpgFromRaw",
    "JpgFromRawStart",
    "JpgFromRawLength",
    "OtherImage",
    "OtherImageStart",
    "OtherImageLength",
    "RawImageSegmentation",
    "DustRemovalData",
    "NEFLinearizationTable",
    "DataDump",
    "SR2SubIFDOffset",
    "SR2SubIFDLength",
    "SR2SubIFDKey",
    "SonyToneCurve",
    "TiffMeteringImage",
    // XMP metadata (we don't parse XMP sidecar data)
    "Rating",
    "RatingPercent",
    "Prefs",
    "Tagged",
    // ICC Profile data (embedded color profiles)
    "ProfileCopyright",
    "ProfileDateTime",
    "ProfileFileSignature",
    "ProfileClass",
    "ProfileCreator",
    "ProfileDescription",
    "ProfileVersion",
    // Crop/output fields (ExifTool calculates these)
    "CropOutputPixels",
    "CropOutputWidthInches",
    "CropOutputHeightInches",
];

/// Compare JSON outputs from exiftool and fpexif
fn compare_json_outputs(
    exiftool_json: &serde_json::Value,
    fpexif_json: &serde_json::Value,
) -> (
    HashMap<String, TagComparison>,
    Vec<TestIssue>,
    usize,
    usize,
    usize,
    usize,
) {
    let mut tags = HashMap::new();
    let mut issues = Vec::new();
    let mut matching = 0usize;
    let mut mismatched = 0usize;
    let mut missing = 0usize;
    let mut extra = 0usize;

    let exiftool_obj = exiftool_json
        .as_array()
        .and_then(|a| a.first())
        .and_then(|o| o.as_object());

    let fpexif_obj = fpexif_json
        .as_array()
        .and_then(|a| a.first())
        .and_then(|o| o.as_object());

    let (exiftool_obj, fpexif_obj) = match (exiftool_obj, fpexif_obj) {
        (Some(e), Some(f)) => (e, f),
        _ => {
            issues.push(TestIssue {
                category: IssueCategory::ParseError,
                message: "Invalid JSON structure".to_string(),
                field: None,
                expected: None,
                actual: None,
            });
            return (tags, issues, matching, mismatched, missing, extra);
        }
    };

    // Check fields from exiftool
    for (key, exiftool_value) in exiftool_obj {
        if IGNORE_FIELDS.contains(&key.as_str()) {
            continue;
        }

        let exiftool_str = value_to_string(exiftool_value);

        match fpexif_obj.get(key) {
            None => {
                missing += 1;
                tags.insert(
                    key.clone(),
                    TagComparison {
                        tag_name: key.clone(),
                        fpexif_value: None,
                        exiftool_value: Some(exiftool_str.clone()),
                        matches: false,
                    },
                );
                issues.push(TestIssue {
                    category: IssueCategory::MissingField,
                    message: format!("Missing field: {}", key),
                    field: Some(key.clone()),
                    expected: Some(exiftool_str),
                    actual: None,
                });
            }
            Some(fpexif_value) => {
                let fpexif_str = value_to_string(fpexif_value);
                let matches = values_match(exiftool_value, fpexif_value);

                if matches {
                    matching += 1;
                } else {
                    mismatched += 1;
                    issues.push(TestIssue {
                        category: IssueCategory::ValueMismatch,
                        message: format!("Value mismatch for {}", key),
                        field: Some(key.clone()),
                        expected: Some(exiftool_str.clone()),
                        actual: Some(fpexif_str.clone()),
                    });
                }

                tags.insert(
                    key.clone(),
                    TagComparison {
                        tag_name: key.clone(),
                        fpexif_value: Some(fpexif_str),
                        exiftool_value: Some(exiftool_str),
                        matches,
                    },
                );
            }
        }
    }

    // Check for extra fields in fpexif
    for key in fpexif_obj.keys() {
        if !exiftool_obj.contains_key(key) && !IGNORE_FIELDS.contains(&key.as_str()) {
            extra += 1;
            let fpexif_str = value_to_string(&fpexif_obj[key]);
            tags.insert(
                key.clone(),
                TagComparison {
                    tag_name: key.clone(),
                    fpexif_value: Some(fpexif_str),
                    exiftool_value: None,
                    matches: false,
                },
            );
            issues.push(TestIssue {
                category: IssueCategory::ExtraField,
                message: format!("Extra field in fpexif: {}", key),
                field: Some(key.clone()),
                expected: None,
                actual: None,
            });
        }
    }

    (tags, issues, matching, mismatched, missing, extra)
}

/// Convert a JSON value to a display string
fn value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        _ => value.to_string(),
    }
}

/// Check if two JSON values match (with tolerance for numeric differences)
fn values_match(a: &serde_json::Value, b: &serde_json::Value) -> bool {
    // For numeric values, allow small floating point differences
    if let (Some(a_num), Some(b_num)) = (a.as_f64(), b.as_f64()) {
        return (a_num - b_num).abs() < 0.001;
    }

    // For strings, compare trimmed values
    if let (Some(a_str), Some(b_str)) = (a.as_str(), b.as_str()) {
        return a_str.trim() == b_str.trim();
    }

    // Handle mixed number/string comparisons (e.g., FirmwareVersion "1.02" vs 1.02)
    // ExifTool sometimes outputs numeric-looking values as numbers
    if let Some(a_str) = a.as_str() {
        if let Some(b_num) = b.as_f64() {
            if let Ok(a_num) = a_str.trim().parse::<f64>() {
                return (a_num - b_num).abs() < 0.001;
            }
        }
    }
    if let Some(b_str) = b.as_str() {
        if let Some(a_num) = a.as_f64() {
            if let Ok(b_num) = b_str.trim().parse::<f64>() {
                return (a_num - b_num).abs() < 0.001;
            }
        }
    }

    // Otherwise compare directly
    a == b
}

/// Find all test files for given formats (searches recursively)
fn find_test_files(formats: &[&str]) -> Vec<String> {
    let test_dir = get_test_files_dir();
    let mut files = Vec::new();

    fn walk_dir(dir: &Path, formats: &[&str], files: &mut Vec<String>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk_dir(&path, formats, files);
                } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    for format in formats {
                        if ext.eq_ignore_ascii_case(format) {
                            if let Some(path_str) = path.to_str() {
                                files.push(path_str.to_string());
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    walk_dir(Path::new(&test_dir), formats, &mut files);
    files.sort();
    files
}

/// Run exiftool comparison for a manufacturer
pub fn run_exiftool_comparison(
    manufacturer: &str,
    formats: &[&str],
    verbose: bool,
) -> Result<ManufacturerTestResult, String> {
    if !test_files_exist() {
        return Err(format!(
            "Test files directory '{}' not found. Set FPEXIF_TEST_FILES environment variable.",
            get_test_files_dir()
        ));
    }

    if !exiftool_available() {
        return Err("exiftool is not available. Please install exiftool.".to_string());
    }

    let format_strings: Vec<String> = formats.iter().map(|s| s.to_string()).collect();
    let mut result = ManufacturerTestResult::new(manufacturer, format_strings);

    let test_files = find_test_files(formats);

    if test_files.is_empty() {
        return Err(format!("No test files found for formats: {:?}", formats));
    }

    if verbose {
        eprintln!(
            "Found {} files to test for {}",
            test_files.len(),
            manufacturer
        );
    }

    for file_path in test_files {
        if verbose {
            eprintln!("Testing: {}", file_path);
        }

        let file_name = Path::new(&file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&file_path)
            .to_string();

        let format = Path::new(&file_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_uppercase();

        // Get outputs from both tools
        let exiftool_json = match get_exiftool_json(&file_path) {
            Ok(j) => j,
            Err(e) => {
                if verbose {
                    eprintln!("  Skipping (exiftool error): {}", e);
                }
                continue;
            }
        };

        let fpexif_json = match get_fpexif_json(&file_path) {
            Ok(j) => j,
            Err(e) => {
                let file_result = FileTestResult {
                    file_path: file_path.clone(),
                    file_name,
                    format,
                    success: false,
                    fpexif_tag_count: 0,
                    exiftool_tag_count: 0,
                    matching_tags: 0,
                    mismatched_tags: 0,
                    missing_tags: 0,
                    extra_tags: 0,
                    tags: HashMap::new(),
                    issues: vec![TestIssue {
                        category: IssueCategory::Critical,
                        message: format!("fpexif failed: {}", e),
                        field: None,
                        expected: None,
                        actual: None,
                    }],
                };
                result.add_file_result(file_result);
                continue;
            }
        };

        // Compare outputs
        let (tags, issues, matching, mismatched, missing, extra) =
            compare_json_outputs(&exiftool_json, &fpexif_json);

        // Count tags
        let exiftool_tag_count = exiftool_json
            .as_array()
            .and_then(|a| a.first())
            .and_then(|o| o.as_object())
            .map(|m| m.len())
            .unwrap_or(0);

        let fpexif_tag_count = fpexif_json
            .as_array()
            .and_then(|a| a.first())
            .and_then(|o| o.as_object())
            .map(|m| m.len())
            .unwrap_or(0);

        let file_result = FileTestResult {
            file_path: file_path.clone(),
            file_name,
            format,
            success: issues
                .iter()
                .all(|i| !matches!(i.category, IssueCategory::Critical)),
            fpexif_tag_count,
            exiftool_tag_count,
            matching_tags: matching,
            mismatched_tags: mismatched,
            missing_tags: missing,
            extra_tags: extra,
            tags,
            issues,
        };

        result.add_file_result(file_result);
    }

    Ok(result)
}

/// Compare current results against a baseline
pub fn compare_with_baseline(
    current: &ManufacturerTestResult,
    baseline: &Baseline,
) -> BaselineDiff {
    let mut improvements = Vec::new();
    let mut regressions = Vec::new();
    let mut new_files = Vec::new();
    let mut removed_files = Vec::new();

    // Build maps for easy lookup
    let baseline_files: HashMap<_, _> = baseline
        .result
        .file_results
        .iter()
        .map(|f| (f.file_path.clone(), f))
        .collect();

    let current_files: HashMap<_, _> = current
        .file_results
        .iter()
        .map(|f| (f.file_path.clone(), f))
        .collect();

    // Check for improvements and regressions
    for current_file in &current.file_results {
        match baseline_files.get(&current_file.file_path) {
            None => {
                new_files.push(current_file.file_name.clone());
            }
            Some(baseline_file) => {
                // Compare tags
                for (tag_name, current_tag) in &current_file.tags {
                    if let Some(baseline_tag) = baseline_file.tags.get(tag_name) {
                        // Tag exists in both - check for changes
                        if current_tag.matches && !baseline_tag.matches {
                            // Improvement: was wrong, now correct
                            improvements.push(TagChange {
                                file: current_file.file_name.clone(),
                                tag: tag_name.clone(),
                                was: baseline_tag
                                    .fpexif_value
                                    .clone()
                                    .unwrap_or_else(|| "(missing)".to_string()),
                                now: current_tag
                                    .fpexif_value
                                    .clone()
                                    .unwrap_or_else(|| "(missing)".to_string()),
                            });
                        } else if !current_tag.matches && baseline_tag.matches {
                            // Regression: was correct, now wrong
                            regressions.push(TagChange {
                                file: current_file.file_name.clone(),
                                tag: tag_name.clone(),
                                was: baseline_tag
                                    .fpexif_value
                                    .clone()
                                    .unwrap_or_else(|| "(missing)".to_string()),
                                now: current_tag
                                    .fpexif_value
                                    .clone()
                                    .unwrap_or_else(|| "(missing)".to_string()),
                            });
                        }
                    } else if current_tag.matches {
                        // New tag that matches - improvement
                        improvements.push(TagChange {
                            file: current_file.file_name.clone(),
                            tag: tag_name.clone(),
                            was: "(not tested)".to_string(),
                            now: current_tag
                                .fpexif_value
                                .clone()
                                .unwrap_or_else(|| "(present)".to_string()),
                        });
                    }
                }
            }
        }
    }

    // Check for removed files
    for baseline_file in &baseline.result.file_results {
        if !current_files.contains_key(&baseline_file.file_path) {
            removed_files.push(baseline_file.file_name.clone());
        }
    }

    BaselineDiff {
        baseline_commit: baseline.metadata.git_commit.clone(),
        baseline_date: baseline.metadata.created_at.clone(),
        current_commit: get_git_commit(),
        matching_delta: current.total_matching_tags as i64
            - baseline.result.total_matching_tags as i64,
        mismatched_delta: current.total_mismatched_tags as i64
            - baseline.result.total_mismatched_tags as i64,
        missing_delta: current.total_missing_tags as i64
            - baseline.result.total_missing_tags as i64,
        extra_delta: current.total_extra_tags as i64 - baseline.result.total_extra_tags as i64,
        improvements,
        regressions,
        new_files,
        removed_files,
    }
}

// =============================================================================
// exiv2 comparison support
// =============================================================================

/// Check if exiv2 is available
pub fn exiv2_available() -> bool {
    Command::new("exiv2").arg("--version").output().is_ok()
}

/// Parse exiv2 output line into (key, type, count, value)
fn parse_exiv2_line(line: &str) -> Option<(String, String, String, String)> {
    // exiv2 format: "Exif.Image.Make                              Ascii       6  Canon"
    // The format is: key (variable width), type, count, value
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 4 {
        let key = parts[0].to_string();
        let type_name = parts[1].to_string();
        let count = parts[2].to_string();
        let value = parts[3..].join(" ");
        Some((key, type_name, count, value))
    } else {
        None
    }
}

/// Get exiv2 output for a file
/// Uses -Pkycv flags: Key, tYpe, Count, raw Value (untranslated)
fn get_exiv2_output(path: &str) -> Result<Vec<(String, String, String, String)>, String> {
    let output = Command::new("exiv2")
        .args(["-Pkycv", path])
        .output()
        .map_err(|e| format!("Failed to run exiv2: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "exiv2 failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().filter_map(parse_exiv2_line).collect())
}

/// Get fpexif exiv2 output for a file
fn get_fpexif_exiv2_output(path: &str) -> Result<Vec<(String, String, String, String)>, String> {
    let output = Command::new("cargo")
        .args(["run", "--features", "cli", "--bin", "fpexif", "--"])
        .args(["exiv2", path])
        .output()
        .map_err(|e| format!("Failed to run fpexif: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "fpexif failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().filter_map(parse_exiv2_line).collect())
}

/// Compare exiv2 outputs and return results
fn compare_exiv2_outputs(
    exiv2_output: &[(String, String, String, String)],
    fpexif_output: &[(String, String, String, String)],
) -> (
    HashMap<String, TagComparison>,
    Vec<TestIssue>,
    usize,
    usize,
    usize,
    usize,
) {
    let mut tags = HashMap::new();
    let mut issues = Vec::new();
    let mut matching = 0usize;
    let mut mismatched = 0usize;
    let mut missing = 0usize;
    let mut extra = 0usize;

    // Create maps for easier lookup
    let exiv2_map: HashMap<_, _> = exiv2_output
        .iter()
        .map(|(k, t, c, v)| (k.clone(), (t.clone(), c.clone(), v.clone())))
        .collect();

    let fpexif_map: HashMap<_, _> = fpexif_output
        .iter()
        .map(|(k, t, c, v)| (k.clone(), (t.clone(), c.clone(), v.clone())))
        .collect();

    // Check fields from exiv2
    for (key, (exiv2_type, exiv2_count, exiv2_value)) in &exiv2_map {
        match fpexif_map.get(key) {
            None => {
                missing += 1;
                tags.insert(
                    key.clone(),
                    TagComparison {
                        tag_name: key.clone(),
                        fpexif_value: None,
                        exiftool_value: Some(exiv2_value.clone()),
                        matches: false,
                    },
                );
                issues.push(TestIssue {
                    category: IssueCategory::MissingField,
                    message: format!("Missing field: {}", key),
                    field: Some(key.clone()),
                    expected: Some(exiv2_value.clone()),
                    actual: None,
                });
            }
            Some((fpexif_type, fpexif_count, fpexif_value)) => {
                // Check if values match (we don't strictly check type/count for exiv2)
                let values_match = exiv2_value.trim() == fpexif_value.trim();

                if values_match {
                    matching += 1;
                } else {
                    mismatched += 1;
                    issues.push(TestIssue {
                        category: IssueCategory::ValueMismatch,
                        message: format!("Value mismatch for {}", key),
                        field: Some(key.clone()),
                        expected: Some(exiv2_value.clone()),
                        actual: Some(fpexif_value.clone()),
                    });
                }

                // Also check for type mismatches (informational)
                if exiv2_type != fpexif_type {
                    issues.push(TestIssue {
                        category: IssueCategory::TypeMismatch,
                        message: format!(
                            "Type mismatch for {}: exiv2={} fpexif={}",
                            key, exiv2_type, fpexif_type
                        ),
                        field: Some(key.clone()),
                        expected: Some(exiv2_type.clone()),
                        actual: Some(fpexif_type.clone()),
                    });
                }

                // Also check for count mismatches (informational)
                if exiv2_count != fpexif_count {
                    issues.push(TestIssue {
                        category: IssueCategory::ExtraField, // Using ExtraField for count mismatch info
                        message: format!(
                            "Count mismatch for {}: exiv2={} fpexif={}",
                            key, exiv2_count, fpexif_count
                        ),
                        field: Some(key.clone()),
                        expected: Some(exiv2_count.clone()),
                        actual: Some(fpexif_count.clone()),
                    });
                }

                tags.insert(
                    key.clone(),
                    TagComparison {
                        tag_name: key.clone(),
                        fpexif_value: Some(fpexif_value.clone()),
                        exiftool_value: Some(exiv2_value.clone()),
                        matches: values_match,
                    },
                );
            }
        }
    }

    // Check for extra fields in fpexif
    for key in fpexif_map.keys() {
        if !exiv2_map.contains_key(key) {
            extra += 1;
            let fpexif_val = &fpexif_map[key].2;
            tags.insert(
                key.clone(),
                TagComparison {
                    tag_name: key.clone(),
                    fpexif_value: Some(fpexif_val.clone()),
                    exiftool_value: None,
                    matches: false,
                },
            );
            issues.push(TestIssue {
                category: IssueCategory::ExtraField,
                message: format!("Extra field in fpexif: {}", key),
                field: Some(key.clone()),
                expected: None,
                actual: None,
            });
        }
    }

    (tags, issues, matching, mismatched, missing, extra)
}

/// Run exiv2 comparison for a manufacturer
pub fn run_exiv2_comparison(
    manufacturer: &str,
    formats: &[&str],
    verbose: bool,
) -> Result<ManufacturerTestResult, String> {
    if !test_files_exist() {
        return Err(format!(
            "Test files directory '{}' not found. Set FPEXIF_TEST_FILES environment variable.",
            get_test_files_dir()
        ));
    }

    if !exiv2_available() {
        return Err("exiv2 is not available. Please install exiv2.".to_string());
    }

    let format_strings: Vec<String> = formats.iter().map(|s| s.to_string()).collect();
    let mut result = ManufacturerTestResult::new(manufacturer, format_strings);

    let test_files = find_test_files(formats);

    if test_files.is_empty() {
        return Err(format!("No test files found for formats: {:?}", formats));
    }

    if verbose {
        eprintln!(
            "Found {} files to test for {} (vs exiv2)",
            test_files.len(),
            manufacturer
        );
    }

    for file_path in test_files {
        if verbose {
            eprintln!("Testing: {}", file_path);
        }

        let file_name = Path::new(&file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&file_path)
            .to_string();

        let format = Path::new(&file_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_uppercase();

        // Get outputs from both tools
        let exiv2_output = match get_exiv2_output(&file_path) {
            Ok(o) => o,
            Err(e) => {
                if verbose {
                    eprintln!("  Skipping (exiv2 error): {}", e);
                }
                continue;
            }
        };

        let fpexif_output = match get_fpexif_exiv2_output(&file_path) {
            Ok(o) => o,
            Err(e) => {
                let file_result = FileTestResult {
                    file_path: file_path.clone(),
                    file_name,
                    format,
                    success: false,
                    fpexif_tag_count: 0,
                    exiftool_tag_count: exiv2_output.len(),
                    matching_tags: 0,
                    mismatched_tags: 0,
                    missing_tags: 0,
                    extra_tags: 0,
                    tags: HashMap::new(),
                    issues: vec![TestIssue {
                        category: IssueCategory::Critical,
                        message: format!("fpexif failed: {}", e),
                        field: None,
                        expected: None,
                        actual: None,
                    }],
                };
                result.add_file_result(file_result);
                continue;
            }
        };

        // Compare outputs
        let (tags, issues, matching, mismatched, missing, extra) =
            compare_exiv2_outputs(&exiv2_output, &fpexif_output);

        let file_result = FileTestResult {
            file_path: file_path.clone(),
            file_name,
            format,
            success: issues
                .iter()
                .all(|i| !matches!(i.category, IssueCategory::Critical)),
            fpexif_tag_count: fpexif_output.len(),
            exiftool_tag_count: exiv2_output.len(),
            matching_tags: matching,
            mismatched_tags: mismatched,
            missing_tags: missing,
            extra_tags: extra,
            tags,
            issues,
        };

        result.add_file_result(file_result);
    }

    Ok(result)
}
