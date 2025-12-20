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

    // Otherwise compare directly
    a == b
}

/// Find all test files for given formats
fn find_test_files(formats: &[&str]) -> Vec<String> {
    let test_dir = get_test_files_dir();
    let mut files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&test_dir) {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                for format in formats {
                    if ext.eq_ignore_ascii_case(format) {
                        if let Some(path) = entry.path().to_str() {
                            files.push(path.to_string());
                        }
                        break;
                    }
                }
            }
        }
    }

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
