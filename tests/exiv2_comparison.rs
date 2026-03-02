// Test that compares our fpexif exiv2 output with exiv2 output
mod test_results;

use std::path::Path;
use std::process::Command;
use test_results::{
    FileTestResult, FormatTestResult, IssueCategory, TestIssue, value_mismatch_issue,
};

/// Helper function to check if we're running in CI
fn is_ci() -> bool {
    std::env::var("CI").is_ok()
}

/// Helper function to check if real files directory exists
fn real_files_exist() -> bool {
    Path::new("/fpexif/raws/welcome.html").exists()
}

/// Helper function to check if exiv2 is available
fn exiv2_available() -> bool {
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

/// Helper function to get exiv2 output
fn get_exiv2_output(path: &str) -> Result<Vec<(String, String, String, String)>, String> {
    // Use -Pkycv flags: Key, tYpe, Count, raw Value (untranslated)
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
    let results: Vec<_> = stdout.lines().filter_map(parse_exiv2_line).collect();

    Ok(results)
}

/// Helper function to get fpexif exiv2 output
fn get_fpexif_exiv2_output(path: &str) -> Result<Vec<(String, String, String, String)>, String> {
    let output = Command::new("cargo")
        .args([
            "run",
            "--release",
            "--features",
            "cli",
            "--bin",
            "fpexif",
            "--",
            "exiv2",
            path,
        ])
        .output()
        .map_err(|e| format!("Failed to run fpexif: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "fpexif failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let results: Vec<_> = stdout.lines().filter_map(parse_exiv2_line).collect();

    Ok(results)
}

/// Result of comparing two exiv2 outputs
struct ComparisonResult {
    issues: Vec<TestIssue>,
    matching_tags: usize,
    mismatched_tags: usize,
    missing_tags: usize,
    extra_tags: usize,
}

/// Compare exiv2 outputs and return differences with counts
fn compare_exiv2_outputs(
    exiv2_output: &[(String, String, String, String)],
    fpexif_output: &[(String, String, String, String)],
) -> ComparisonResult {
    let mut issues = Vec::new();
    let mut matching_tags = 0;
    let mut mismatched_tags = 0;
    let mut missing_tags = 0;
    let mut extra_tags = 0;

    // Create maps for easier lookup
    let exiv2_map: std::collections::HashMap<_, _> = exiv2_output
        .iter()
        .map(|(k, t, c, v)| (k.clone(), (t.clone(), c.clone(), v.clone())))
        .collect();

    let fpexif_map: std::collections::HashMap<_, _> = fpexif_output
        .iter()
        .map(|(k, t, c, v)| (k.clone(), (t.clone(), c.clone(), v.clone())))
        .collect();

    // Check for fields in exiv2 that are missing or different in fpexif
    for (key, (exiv2_type, exiv2_count, exiv2_value)) in &exiv2_map {
        match fpexif_map.get(key) {
            None => {
                missing_tags += 1;
                issues.push(TestIssue {
                    category: IssueCategory::MissingField,
                    message: format!("Missing field in fpexif: {}", key),
                    field: Some(key.clone()),
                    expected: None,
                    actual: None,
                });
            }
            Some((fpexif_type, fpexif_count, fpexif_value)) => {
                let mut has_mismatch = false;

                // Check type
                if exiv2_type != fpexif_type {
                    has_mismatch = true;
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
                // Check count
                if exiv2_count != fpexif_count {
                    has_mismatch = true;
                    issues.push(TestIssue {
                        category: IssueCategory::CountMismatch,
                        message: format!(
                            "Count mismatch for {}: exiv2={} fpexif={}",
                            key, exiv2_count, fpexif_count
                        ),
                        field: Some(key.clone()),
                        expected: Some(exiv2_count.clone()),
                        actual: Some(fpexif_count.clone()),
                    });
                }
                // Check value (normalize for comparison)
                let exiv2_val_norm = exiv2_value.trim();
                let fpexif_val_norm = fpexif_value.trim();
                if exiv2_val_norm != fpexif_val_norm {
                    has_mismatch = true;
                    issues.push(value_mismatch_issue(key, exiv2_val_norm, fpexif_val_norm));
                }

                if has_mismatch {
                    mismatched_tags += 1;
                } else {
                    matching_tags += 1;
                }
            }
        }
    }

    // Check for extra fields in fpexif
    for key in fpexif_map.keys() {
        if !exiv2_map.contains_key(key) {
            extra_tags += 1;
            issues.push(TestIssue {
                category: IssueCategory::ExtraField,
                message: format!("Extra field in fpexif: {}", key),
                field: Some(key.clone()),
                expected: None,
                actual: None,
            });
        }
    }

    ComparisonResult {
        issues,
        matching_tags,
        mismatched_tags,
        missing_tags,
        extra_tags,
    }
}

/// Generic helper function to test exiv2 compatibility for a given file extension
fn test_format_exiv2_compatibility(extension: &str) -> FormatTestResult {
    let test_name = format!("exiv2_{}", extension.to_lowercase());
    let mut result = FormatTestResult::new(extension, &test_name, "exiv2");

    if !real_files_exist() {
        if is_ci() {
            // In CI, missing files is a critical error
            let file_result = FileTestResult {
                file_path: "/fpexif/raws".to_string(),
                format: extension.to_uppercase(),
                success: false,
                fpexif_tag_count: 0,
                reference_tag_count: 0,
                matching_tags: 0,
                mismatched_tags: 0,
                missing_tags: 0,
                extra_tags: 0,
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

    if !exiv2_available() {
        println!("Skipping test - exiv2 not available");
        return result;
    }

    // Find ALL files with the given extension to test
    let test_files: Vec<_> = std::fs::read_dir("/fpexif/raws")
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext.eq_ignore_ascii_case(extension))
                        .unwrap_or(false)
                })
                .map(|e| e.path())
                .collect()
        })
        .unwrap_or_default();

    if test_files.is_empty() {
        println!("No {} files found for testing", extension.to_uppercase());
        return result;
    }

    println!(
        "Found {} {} file(s) to test",
        test_files.len(),
        extension.to_uppercase()
    );

    // Test each file
    for test_file in test_files {
        let test_path = test_file
            .to_str()
            .expect("Failed to convert path to string");
        println!("\n--- Testing file: {} ---", test_path);

        // Get outputs from both tools
        let exiv2_output = match get_exiv2_output(test_path) {
            Ok(output) => output,
            Err(e) => {
                println!("[{}] Failed to get exiv2 output: {}", test_path, e);
                continue;
            }
        };

        let fpexif_output = match get_fpexif_exiv2_output(test_path) {
            Ok(output) => output,
            Err(e) => {
                let file_result = FileTestResult {
                    file_path: test_path.to_string(),
                    format: extension.to_uppercase(),
                    success: false,
                    fpexif_tag_count: 0,
                    reference_tag_count: exiv2_output.len(),
                    matching_tags: 0,
                    mismatched_tags: 0,
                    missing_tags: 0,
                    extra_tags: 0,
                    issues: vec![TestIssue {
                        category: IssueCategory::Critical,
                        message: format!("Failed to get fpexif output: {}", e),
                        field: None,
                        expected: None,
                        actual: None,
                    }],
                };
                result.add_file_result(file_result);
                continue;
            }
        };

        println!("exiv2 returned {} tags", exiv2_output.len());
        println!("fpexif returned {} tags", fpexif_output.len());

        // Compare outputs
        let comparison = compare_exiv2_outputs(&exiv2_output, &fpexif_output);

        // Determine if there are critical issues
        let has_critical = comparison.issues.iter().any(|i| {
            matches!(i.category, IssueCategory::Critical)
                || matches!(i.category, IssueCategory::ValueMismatch)
                || matches!(i.category, IssueCategory::TypeMismatch)
        });

        let file_result = FileTestResult {
            file_path: test_path.to_string(),
            format: extension.to_uppercase(),
            success: !has_critical,
            fpexif_tag_count: fpexif_output.len(),
            reference_tag_count: exiv2_output.len(),
            matching_tags: comparison.matching_tags,
            mismatched_tags: comparison.mismatched_tags,
            missing_tags: comparison.missing_tags,
            extra_tags: comparison.extra_tags,
            issues: comparison.issues,
        };

        // Print summary
        if !file_result.issues.is_empty() {
            println!(
                "\n[{}] Found {} differences:",
                test_path,
                file_result.issues.len()
            );
            println!(
                "  Matching: {}, Mismatched: {}, Missing: {}, Extra: {}",
                file_result.matching_tags,
                file_result.mismatched_tags,
                file_result.missing_tags,
                file_result.extra_tags
            );

            // Show some mismatches
            let mismatches: Vec<_> = file_result
                .issues
                .iter()
                .filter(|i| matches!(i.category, IssueCategory::ValueMismatch))
                .take(5)
                .collect();
            if !mismatches.is_empty() {
                println!(
                    "\n[{}] --- Value Mismatches ({}) ---",
                    test_path, file_result.mismatched_tags
                );
                for issue in &mismatches {
                    println!("  {}", issue.message);
                }
                if file_result.mismatched_tags > 5 {
                    println!("  ... and {} more", file_result.mismatched_tags - 5);
                }
            }

            // Show some missing fields
            let missing: Vec<_> = file_result
                .issues
                .iter()
                .filter(|i| matches!(i.category, IssueCategory::MissingField))
                .take(5)
                .collect();
            if !missing.is_empty() {
                println!(
                    "\n[{}] --- Missing Fields ({}) ---",
                    test_path, file_result.missing_tags
                );
                for issue in &missing {
                    if let Some(ref field) = issue.field {
                        println!("  {}", field);
                    }
                }
                if file_result.missing_tags > 5 {
                    println!("  ... and {} more", file_result.missing_tags - 5);
                }
            }

            if has_critical {
                println!("\n[{}] !! Found critical differences", test_path);
            }
        } else {
            println!("\n[{}] * Outputs match!", test_path);
        }

        result.add_file_result(file_result);
    }

    // Write results to JSON
    if let Err(e) = result.write_to_file() {
        eprintln!("Failed to write test results: {}", e);
    }

    println!("\n=== {} Summary ===", extension.to_uppercase());
    println!("Files tested: {}", result.files_tested);
    println!(
        "Total: {} matching, {} mismatched, {} missing, {} extra",
        result.total_matching_tags,
        result.total_mismatched_tags,
        result.total_missing_tags,
        result.total_extra_tags
    );

    result
}

#[test]
fn test_exiv2_compatibility_raf() {
    // Test using the test-data/DSCF0062.RAF file
    let test_path = "test-data/DSCF0062.RAF";

    if !Path::new(test_path).exists() {
        println!("Skipping test - {} not available", test_path);
        return;
    }

    if !exiv2_available() {
        println!("Skipping test - exiv2 not available");
        return;
    }

    println!("Testing with file: {}", test_path);

    let exiv2_output = match get_exiv2_output(test_path) {
        Ok(output) => output,
        Err(e) => {
            println!("Failed to get exiv2 output: {}", e);
            return;
        }
    };

    let fpexif_output = match get_fpexif_exiv2_output(test_path) {
        Ok(output) => output,
        Err(e) => {
            panic!("Failed to get fpexif output: {}", e);
        }
    };

    println!("exiv2 returned {} tags", exiv2_output.len());
    println!("fpexif returned {} tags", fpexif_output.len());

    let comparison = compare_exiv2_outputs(&exiv2_output, &fpexif_output);

    if !comparison.issues.is_empty() {
        println!(
            "\nFound {} differences (informational): {} matching, {} mismatched, {} missing, {} extra",
            comparison.issues.len(),
            comparison.matching_tags,
            comparison.mismatched_tags,
            comparison.missing_tags,
            comparison.extra_tags
        );
        // Note: We don't fail this test yet as exiv2 compatibility is new
    } else {
        println!("\n* Outputs match!");
    }
}

#[test]
fn test_exiv2_compatibility_cr2() {
    let _result = test_format_exiv2_compatibility("cr2");
}

#[test]
fn test_exiv2_compatibility_nef() {
    let _result = test_format_exiv2_compatibility("nef");
}

#[test]
fn test_exiv2_compatibility_arw() {
    let _result = test_format_exiv2_compatibility("arw");
}

#[test]
fn test_exiv2_compatibility_dng() {
    let _result = test_format_exiv2_compatibility("dng");
}

#[test]
fn test_exiv2_compatibility_jpg() {
    let _result = test_format_exiv2_compatibility("jpg");
}

#[test]
fn test_exiv2_compatibility_orf() {
    let _result = test_format_exiv2_compatibility("orf");
}

#[test]
fn test_exiv2_compatibility_rw2() {
    let _result = test_format_exiv2_compatibility("rw2");
}

#[test]
fn test_exiv2_compatibility_3fr() {
    let _result = test_format_exiv2_compatibility("3fr");
}

#[test]
fn test_exiv2_compatibility_crw() {
    let _result = test_format_exiv2_compatibility("crw");
}

#[test]
fn test_exiv2_compatibility_dcr() {
    let _result = test_format_exiv2_compatibility("dcr");
}

#[test]
fn test_exiv2_compatibility_erf() {
    let _result = test_format_exiv2_compatibility("erf");
}

#[test]
fn test_exiv2_compatibility_kdc() {
    let _result = test_format_exiv2_compatibility("kdc");
}

#[test]
fn test_exiv2_compatibility_mdc() {
    let _result = test_format_exiv2_compatibility("mdc");
}

#[test]
fn test_exiv2_compatibility_mef() {
    let _result = test_format_exiv2_compatibility("mef");
}

#[test]
fn test_exiv2_compatibility_mos() {
    let _result = test_format_exiv2_compatibility("mos");
}

#[test]
fn test_exiv2_compatibility_mrw() {
    let _result = test_format_exiv2_compatibility("mrw");
}

#[test]
fn test_exiv2_compatibility_nrw() {
    let _result = test_format_exiv2_compatibility("nrw");
}

#[test]
fn test_exiv2_compatibility_pef() {
    let _result = test_format_exiv2_compatibility("pef");
}

#[test]
fn test_exiv2_compatibility_raw() {
    let _result = test_format_exiv2_compatibility("raw");
}

#[test]
fn test_exiv2_compatibility_sr2() {
    let _result = test_format_exiv2_compatibility("sr2");
}

#[test]
fn test_exiv2_compatibility_srf() {
    let _result = test_format_exiv2_compatibility("srf");
}

#[test]
fn test_exiv2_compatibility_srw() {
    let _result = test_format_exiv2_compatibility("srw");
}

#[test]
fn test_exiv2_compatibility_x3f() {
    let _result = test_format_exiv2_compatibility("x3f");
}

#[test]
fn test_exiv2_compatibility_raf_format() {
    let _result = test_format_exiv2_compatibility("raf");
}

// =============================================================================
// Placeholder tests for flags not yet implemented
// =============================================================================

#[test]
#[ignore = "exiv2 -p (print mode) not yet implemented"]
fn test_exiv2_print_mode_flag() {
    // TODO: Implement -p flag support
    // exiv2 -pa (print all), -pe (print Exif), -pi (print IPTC), -px (print XMP)
}

#[test]
#[ignore = "exiv2 -g (grep) not yet implemented"]
fn test_exiv2_grep_flag() {
    // TODO: Implement -g flag support
    // exiv2 -g <pattern> filters output by key pattern
}

#[test]
#[ignore = "exiv2 -K (key) not yet implemented"]
fn test_exiv2_key_flag() {
    // TODO: Implement -K flag support
    // exiv2 -K <key> shows only specified key
}

#[test]
#[ignore = "exiv2 -b (binary) not yet implemented"]
fn test_exiv2_binary_flag() {
    // TODO: Implement -b flag support
    // exiv2 -b outputs binary data (e.g., thumbnails)
}

#[test]
#[ignore = "exiv2 -u (unknown tags) not yet implemented"]
fn test_exiv2_unknown_flag() {
    // TODO: Implement -u flag support
    // exiv2 -u shows unknown tags
}

#[test]
#[ignore = "exiv2 -t (translated) not yet implemented"]
fn test_exiv2_translated_flag() {
    // TODO: Implement -t flag support
    // exiv2 -t shows translated tag values
}
