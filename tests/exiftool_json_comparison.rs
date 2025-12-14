// Test that compares our --exiftool-json output directly with exiftool -json output
use std::path::Path;
use std::process::Command;

mod test_results;
use test_results::{
    is_critical_missing_field, missing_field_issue, value_mismatch_issue, FileTestResult,
    FormatTestResult, IssueCategory, TestIssue,
};

/// Helper function to check if we're running in CI
fn is_ci() -> bool {
    std::env::var("CI").is_ok()
}

/// Helper function to check if real files directory exists
fn real_files_exist() -> bool {
    Path::new("/fpexif/raws/welcome.html").exists()
}

/// Helper function to check real files exist or fail in CI
fn require_real_files_or_skip(test_name: &str) {
    if !real_files_exist() {
        if is_ci() {
            panic!(
                "Test '{}' requires /fpexif/raws directory but it was not found. \
                In CI, this directory must be present.",
                test_name
            );
        } else {
            println!("Skipping {} - real files directory not found", test_name);
        }
    }
}

/// Helper function to check if exiftool is available
fn exiftool_available() -> bool {
    Command::new("exiftool").arg("--version").output().is_ok()
}

/// Helper function to get exiftool JSON output
fn get_exiftool_json_output(path: &str) -> Result<serde_json::Value, String> {
    let output = Command::new("exiftool")
        .arg("-json")
        .arg(path)
        .output()
        .map_err(|e| format!("Failed to run exiftool: {}", e))?;

    serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse exiftool JSON: {}", e))
}

/// Helper function to get fpexif --exiftool-json output
fn get_fpexif_exiftool_json_output(path: &str) -> Result<serde_json::Value, String> {
    let output = Command::new("cargo")
        .args([
            "run",
            "--release",
            "--features",
            "cli",
            "--bin",
            "fpexif",
            "--",
        ])
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

/// Compare two JSON values, returning issues found
fn compare_json_outputs(
    exiftool_json: &serde_json::Value,
    fpexif_json: &serde_json::Value,
) -> Vec<TestIssue> {
    let mut issues = Vec::new();

    // Both should be arrays
    let exiftool_array = match exiftool_json.as_array() {
        Some(arr) => arr,
        None => {
            issues.push(TestIssue {
                category: IssueCategory::ParseError,
                message: "exiftool output is not an array".to_string(),
                field: None,
                expected: None,
                actual: None,
            });
            return issues;
        }
    };

    let fpexif_array = match fpexif_json.as_array() {
        Some(arr) => arr,
        None => {
            issues.push(TestIssue {
                category: IssueCategory::ParseError,
                message: "fpexif output is not an array".to_string(),
                field: None,
                expected: None,
                actual: None,
            });
            return issues;
        }
    };

    // Should have same number of elements (typically 1)
    if exiftool_array.len() != fpexif_array.len() {
        issues.push(TestIssue {
            category: IssueCategory::Critical,
            message: format!(
                "Array length mismatch: exiftool={} fpexif={}",
                exiftool_array.len(),
                fpexif_array.len()
            ),
            field: None,
            expected: Some(exiftool_array.len().to_string()),
            actual: Some(fpexif_array.len().to_string()),
        });
        return issues;
    }

    // Compare first object (typically the only one)
    if let (Some(exiftool_obj), Some(fpexif_obj)) =
        (exiftool_array[0].as_object(), fpexif_array[0].as_object())
    {
        // Fields to ignore (exiftool-specific or expected to differ)
        let ignore_fields = [
            "SourceFile",               // Paths may differ
            "ExifToolVersion",          // We don't include this
            "FileName",                 // File metadata we don't include
            "Directory",                // File metadata we don't include
            "FileSize",                 // File metadata we don't include
            "FileModifyDate",           // File metadata we don't include
            "FileAccessDate",           // File metadata we don't include
            "FileInodeChangeDate",      // File metadata we don't include
            "FilePermissions",          // File metadata we don't include
            "FileType",                 // File metadata we don't include
            "FileTypeExtension",        // File metadata we don't include
            "MIMEType",                 // File metadata we don't include
            "CurrentIPTCDigest",        // IPTC data we don't parse yet
            "CodedCharacterSet",        // IPTC data we don't parse yet
            "ApplicationRecordVersion", // IPTC data we don't parse yet
            "XMPToolkit",               // XMP data we don't parse yet
            "MakerNote",                // Raw maker note data - we parse it differently
        ];

        // Check for fields in exiftool that are missing or different in fpexif
        for (key, exiftool_value) in exiftool_obj {
            if ignore_fields.contains(&key.as_str()) {
                continue;
            }

            match fpexif_obj.get(key) {
                None => {
                    issues.push(missing_field_issue(key));
                }
                Some(fpexif_value) => {
                    // For numeric values, allow small floating point differences
                    if let (Some(et_num), Some(fp_num)) =
                        (exiftool_value.as_f64(), fpexif_value.as_f64())
                    {
                        if (et_num - fp_num).abs() > 0.001 {
                            issues.push(value_mismatch_issue(
                                key,
                                &et_num.to_string(),
                                &fp_num.to_string(),
                            ));
                        }
                    }
                    // For strings, compare normalized values
                    else if let (Some(et_str), Some(fp_str)) =
                        (exiftool_value.as_str(), fpexif_value.as_str())
                    {
                        let et_normalized = et_str.trim();
                        let fp_normalized = fp_str.trim();
                        if et_normalized != fp_normalized {
                            issues.push(value_mismatch_issue(key, et_str, fp_str));
                        }
                    }
                    // Different types
                    else if exiftool_value != fpexif_value {
                        issues.push(value_mismatch_issue(
                            key,
                            &exiftool_value.to_string(),
                            &fpexif_value.to_string(),
                        ));
                    }
                }
            }
        }

        // Check for extra fields in fpexif that aren't in exiftool (informational, not an error)
        for key in fpexif_obj.keys() {
            if !exiftool_obj.contains_key(key)
                && !key.starts_with("Canon")
                && !key.starts_with("Nikon")
                && !key.starts_with("Sony")
            {
                issues.push(TestIssue {
                    category: IssueCategory::ExtraField,
                    message: format!("Extra field in fpexif (not in exiftool): {}", key),
                    field: Some(key.clone()),
                    expected: None,
                    actual: None,
                });
            }
        }
    }

    issues
}

/// Generic helper function to test exiftool JSON compatibility for a given file extension
fn test_format_exiftool_json_compatibility(extension: &str) -> FormatTestResult {
    let test_name = format!("exiftool_json_{}", extension.to_lowercase());
    let mut result = FormatTestResult::new(extension, &test_name, "exiftool");

    if !real_files_exist() {
        if is_ci() {
            // In CI, missing files is a critical error
            let file_result = FileTestResult {
                file_path: "/fpexif/raws".to_string(),
                format: extension.to_uppercase(),
                success: false,
                fpexif_tag_count: 0,
                reference_tag_count: 0,
                issues: vec![TestIssue {
                    category: IssueCategory::Critical,
                    message: "Test files directory not found in CI".to_string(),
                    field: None,
                    expected: None,
                    actual: None,
                }],
            };
            result.add_file_result(file_result);
        }
        return result;
    }

    if !exiftool_available() {
        println!("Skipping test - exiftool not available");
        return result;
    }

    // Find a file with the given extension to test
    let test_files = std::fs::read_dir("/fpexif/raws").ok().and_then(|entries| {
        entries.filter_map(|e| e.ok()).find(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case(extension))
                .unwrap_or(false)
        })
    });

    let test_file = match test_files {
        Some(entry) => entry.path(),
        None => {
            println!("No {} files found for testing", extension.to_uppercase());
            return result;
        }
    };

    let test_path = test_file
        .to_str()
        .expect("Failed to convert path to string");
    println!("Testing with file: {}", test_path);

    // Get outputs from both tools
    let exiftool_json = match get_exiftool_json_output(test_path) {
        Ok(json) => json,
        Err(e) => {
            println!("Failed to get exiftool output: {}", e);
            return result;
        }
    };

    let fpexif_json = match get_fpexif_exiftool_json_output(test_path) {
        Ok(json) => json,
        Err(e) => {
            let file_result = FileTestResult {
                file_path: test_path.to_string(),
                format: extension.to_uppercase(),
                success: false,
                fpexif_tag_count: 0,
                reference_tag_count: 0,
                issues: vec![TestIssue {
                    category: IssueCategory::Critical,
                    message: format!("Failed to get fpexif output: {}", e),
                    field: None,
                    expected: None,
                    actual: None,
                }],
            };
            result.add_file_result(file_result);
            return result;
        }
    };

    // Compare outputs
    let issues = compare_json_outputs(&exiftool_json, &fpexif_json);

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

    // Determine if there are critical issues
    let has_critical = issues.iter().any(|i| {
        matches!(i.category, IssueCategory::Critical)
            || (matches!(i.category, IssueCategory::ValueMismatch))
            || (matches!(i.category, IssueCategory::MissingField)
                && i.field
                    .as_ref()
                    .map(|f| is_critical_missing_field(f))
                    .unwrap_or(false))
    });

    let file_result = FileTestResult {
        file_path: test_path.to_string(),
        format: extension.to_uppercase(),
        success: !has_critical,
        fpexif_tag_count,
        reference_tag_count: exiftool_tag_count,
        issues,
    };

    // Print summary
    if !file_result.issues.is_empty() {
        println!(
            "\n[{}] Found {} differences:",
            test_path,
            file_result.issues.len()
        );

        let missing: Vec<_> = file_result
            .issues
            .iter()
            .filter(|i| matches!(i.category, IssueCategory::MissingField))
            .collect();
        let mismatches: Vec<_> = file_result
            .issues
            .iter()
            .filter(|i| matches!(i.category, IssueCategory::ValueMismatch))
            .collect();
        let extras: Vec<_> = file_result
            .issues
            .iter()
            .filter(|i| matches!(i.category, IssueCategory::ExtraField))
            .collect();

        if !missing.is_empty() {
            println!(
                "\n[{}] --- Missing Fields ({}) ---",
                test_path,
                missing.len()
            );
            for issue in missing.iter().take(10) {
                println!("  {}", issue.message);
            }
            if missing.len() > 10 {
                println!("  ... and {} more", missing.len() - 10);
            }
        }

        if !mismatches.is_empty() {
            println!(
                "\n[{}] --- Value Mismatches ({}) ---",
                test_path,
                mismatches.len()
            );
            for issue in &mismatches {
                println!("  {}", issue.message);
            }
        }

        if !extras.is_empty() {
            println!("\n[{}] --- Extra Fields ({}) ---", test_path, extras.len());
            for issue in extras.iter().take(5) {
                println!("  {}", issue.message);
            }
            if extras.len() > 5 {
                println!("  ... and {} more", extras.len() - 5);
            }
        }

        if has_critical {
            println!("\n[{}] !! Found critical differences", test_path);
        } else {
            println!("\n[{}] * No critical differences found!", test_path);
        }
    } else {
        println!("\n[{}] * JSON outputs match perfectly!", test_path);
    }

    result.add_file_result(file_result);
    result
}

/// Write result to JSON and return whether test passed
fn run_and_report(extension: &str) {
    require_real_files_or_skip(&format!(
        "test_exiftool_json_compatibility_{}",
        extension.to_lowercase()
    ));
    if !real_files_exist() {
        return;
    }

    let result = test_format_exiftool_json_compatibility(extension);

    // Write JSON result
    if let Err(e) = result.write_to_file() {
        eprintln!("Failed to write test results: {}", e);
    }

    // Fail test if there are critical issues
    if result.has_critical_failures() {
        panic!(
            "Found {} critical issues in {} test",
            result.critical_issues,
            extension.to_uppercase()
        );
    }
}

#[test]
fn test_exiftool_json_compatibility_cr2() {
    run_and_report("cr2");
}

#[test]
fn test_exiftool_json_compatibility_nef() {
    run_and_report("nef");
}

#[test]
fn test_exiftool_json_compatibility_arw() {
    run_and_report("arw");
}

#[test]
fn test_exiftool_json_compatibility_orf() {
    run_and_report("orf");
}

#[test]
fn test_exiftool_json_compatibility_dng() {
    run_and_report("dng");
}

#[test]
fn test_exiftool_json_compatibility_rw2() {
    run_and_report("rw2");
}

#[test]
fn test_exiftool_json_compatibility_dscf_raf() {
    // Test using the test-data/DSCF0062.RAF file
    let test_path = "test-data/DSCF0062.RAF";
    let test_name = "exiftool_json_raf";
    let mut result = FormatTestResult::new("RAF", test_name, "exiftool");

    if !Path::new(test_path).exists() {
        println!("Skipping test - {} not available", test_path);
        return;
    }

    if !exiftool_available() {
        println!("Skipping test - exiftool not available");
        return;
    }

    println!("Testing with file: {}", test_path);

    // Get outputs from both tools
    let exiftool_json = match get_exiftool_json_output(test_path) {
        Ok(json) => json,
        Err(e) => {
            let file_result = FileTestResult {
                file_path: test_path.to_string(),
                format: "RAF".to_string(),
                success: false,
                fpexif_tag_count: 0,
                reference_tag_count: 0,
                issues: vec![TestIssue {
                    category: IssueCategory::Critical,
                    message: format!("Failed to get exiftool output: {}", e),
                    field: None,
                    expected: None,
                    actual: None,
                }],
            };
            result.add_file_result(file_result);
            if let Err(e) = result.write_to_file() {
                eprintln!("Failed to write test results: {}", e);
            }
            panic!("Failed to get exiftool output: {}", e);
        }
    };

    let fpexif_json = match get_fpexif_exiftool_json_output(test_path) {
        Ok(json) => json,
        Err(e) => {
            let file_result = FileTestResult {
                file_path: test_path.to_string(),
                format: "RAF".to_string(),
                success: false,
                fpexif_tag_count: 0,
                reference_tag_count: 0,
                issues: vec![TestIssue {
                    category: IssueCategory::Critical,
                    message: format!("Failed to get fpexif output: {}", e),
                    field: None,
                    expected: None,
                    actual: None,
                }],
            };
            result.add_file_result(file_result);
            if let Err(e) = result.write_to_file() {
                eprintln!("Failed to write test results: {}", e);
            }
            panic!("Failed to get fpexif output: {}", e);
        }
    };

    // Save outputs for manual inspection if needed
    println!("\n=== Exiftool Output Sample ===");
    if let Some(arr) = exiftool_json.as_array() {
        if let Some(obj) = arr.first() {
            if let Some(obj_map) = obj.as_object() {
                let sample_keys: Vec<_> = obj_map.keys().take(5).collect();
                println!("Sample keys (first 5): {:?}", sample_keys);
                println!("Total keys: {}", obj_map.len());
            }
        }
    }

    println!("\n=== Fpexif Output Sample ===");
    if let Some(arr) = fpexif_json.as_array() {
        if let Some(obj) = arr.first() {
            if let Some(obj_map) = obj.as_object() {
                let sample_keys: Vec<_> = obj_map.keys().take(5).collect();
                println!("Sample keys (first 5): {:?}", sample_keys);
                println!("Total keys: {}", obj_map.len());
            }
        }
    }

    // Compare outputs
    let issues = compare_json_outputs(&exiftool_json, &fpexif_json);

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

    // Determine if there are critical issues
    let has_critical = issues.iter().any(|i| {
        matches!(i.category, IssueCategory::Critical)
            || (matches!(i.category, IssueCategory::ValueMismatch))
            || (matches!(i.category, IssueCategory::MissingField)
                && i.field
                    .as_ref()
                    .map(|f| is_critical_missing_field(f))
                    .unwrap_or(false))
    });

    let file_result = FileTestResult {
        file_path: test_path.to_string(),
        format: "RAF".to_string(),
        success: !has_critical,
        fpexif_tag_count,
        reference_tag_count: exiftool_tag_count,
        issues: issues.clone(),
    };

    if !issues.is_empty() {
        println!("\n[{}] Found {} differences:", test_path, issues.len());

        let missing: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.category, IssueCategory::MissingField))
            .collect();
        let extras: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.category, IssueCategory::ExtraField))
            .collect();
        let mismatches: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.category, IssueCategory::ValueMismatch))
            .collect();

        if !missing.is_empty() {
            println!(
                "\n[{}] --- Missing Fields ({}) ---",
                test_path,
                missing.len()
            );
            for issue in &missing {
                println!("  {}", issue.message);
            }
        }

        if !mismatches.is_empty() {
            println!(
                "\n[{}] --- Value Mismatches ({}) ---",
                test_path,
                mismatches.len()
            );
            for issue in &mismatches {
                println!("  {}", issue.message);
            }
        }

        if !extras.is_empty() {
            println!("\n[{}] --- Extra Fields ({}) ---", test_path, extras.len());
            for issue in extras.iter().take(10) {
                println!("  {}", issue.message);
            }
            if extras.len() > 10 {
                println!("  ... and {} more", extras.len() - 10);
            }
        }

        if has_critical {
            println!("\n[{}] !! Found critical differences", test_path);
        } else {
            println!("\n[{}] * No critical differences found!", test_path);
            println!(
                "  (Found {} expected variations from exiftool)",
                issues.len()
            );
        }
    } else {
        println!("\n[{}] * JSON outputs match perfectly!", test_path);
    }

    result.add_file_result(file_result);

    // Write JSON result
    if let Err(e) = result.write_to_file() {
        eprintln!("Failed to write test results: {}", e);
    }

    // Fail test if there are critical issues
    if result.has_critical_failures() {
        panic!(
            "Found {} critical differences in JSON output",
            result.critical_issues
        );
    }
}
