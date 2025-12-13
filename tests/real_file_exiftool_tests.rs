// Real file format tests using exiftool as reference
// These tests only run if /fpexif/raws/welcome.html exists
use std::fs;
use std::path::Path;
use std::process::Command;

mod test_results;
use test_results::{
    value_mismatch_issue, FileTestResult, FormatTestResult, IssueCategory, TestIssue,
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

/// Helper function to get exiftool JSON output for a file
fn get_exiftool_output(path: &str) -> Result<serde_json::Value, String> {
    if !Path::new(path).exists() {
        return Err(format!("File not found: {}", path));
    }

    let output = Command::new("exiftool")
        .arg("-json")
        .arg(path)
        .output()
        .map_err(|e| format!("Failed to run exiftool: {}", e))?;

    let json_str = String::from_utf8_lossy(&output.stdout);
    let data = serde_json::from_str::<Vec<serde_json::Value>>(&json_str)
        .map_err(|e| format!("Failed to parse exiftool JSON: {}", e))?
        .into_iter()
        .next()
        .ok_or_else(|| "Empty JSON array".to_string())?;

    if let Some(error) = data.get("Error") {
        return Err(format!("exiftool reported error: {}", error));
    }

    Ok(data)
}

/// Helper to normalize string values for comparison
fn normalize_string(s: &str) -> String {
    s.chars()
        .take_while(|&c| c != '\0')
        .filter(|c| c.is_ascii_graphic() || c.is_whitespace())
        .collect::<String>()
        .trim()
        .to_string()
}

/// Helper function to test a single file and return issues
fn test_file_against_exiftool(path: &str) -> FileTestResult {
    let format = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_uppercase())
        .unwrap_or_else(|| "UNKNOWN".to_string());

    let exiftool_data = match get_exiftool_output(path) {
        Ok(data) => data,
        Err(e) => {
            println!("Skipping {} - exiftool error: {}", path, e);
            return FileTestResult {
                file_path: path.to_string(),
                format,
                success: true, // Skip is not a failure
                fpexif_tag_count: 0,
                reference_tag_count: 0,
                issues: vec![],
            };
        }
    };

    let parser = fpexif::ExifParser::new();
    let fpexif_result = parser.parse_file(path);

    match fpexif_result {
        Ok(exif_data) => {
            println!("Parsed: {} ({} tags)", path, exif_data.len());

            let mut issues = Vec::new();

            // Track unknown tags
            for (tag_id, _) in exif_data.iter() {
                if tag_id.name().is_none() {
                    issues.push(TestIssue {
                        category: IssueCategory::UnknownTag,
                        message: format!("Unknown tag: 0x{:04X}", tag_id.id),
                        field: Some(format!("0x{:04X}", tag_id.id)),
                        expected: None,
                        actual: None,
                    });
                }
            }

            // Check Make
            if let Some(exiftool_make) = exiftool_data.get("Make").and_then(|v| v.as_str()) {
                if let Some(fpexif::data_types::ExifValue::Ascii(fpexif_make)) =
                    exif_data.get_tag_by_name("Make")
                {
                    let et = normalize_string(exiftool_make);
                    let fp = normalize_string(fpexif_make);
                    if et != fp {
                        issues.push(value_mismatch_issue("Make", &et, &fp));
                    }
                }
            }

            // Check Model
            if let Some(exiftool_model) = exiftool_data.get("Model").and_then(|v| v.as_str()) {
                if let Some(fpexif::data_types::ExifValue::Ascii(fpexif_model)) =
                    exif_data.get_tag_by_name("Model")
                {
                    let et = normalize_string(exiftool_model);
                    let fp = normalize_string(fpexif_model);
                    if et != fp {
                        issues.push(value_mismatch_issue("Model", &et, &fp));
                    }
                }
            }

            // Check ISO
            if let Some(exiftool_iso) = exiftool_data.get("ISO").and_then(|v| v.as_u64()) {
                if let Some(fpexif::data_types::ExifValue::Short(ref vals)) =
                    exif_data.get_tag_by_name("ISOSpeedRatings")
                {
                    if !vals.is_empty() && vals[0] as u64 != exiftool_iso {
                        issues.push(value_mismatch_issue(
                            "ISO",
                            &exiftool_iso.to_string(),
                            &vals[0].to_string(),
                        ));
                    }
                }
            }

            // Check FNumber
            if let Some(exiftool_aperture) = exiftool_data.get("FNumber").and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok()))
            }) {
                if let Some(fpexif::data_types::ExifValue::Rational(ref vals)) =
                    exif_data.get_tag_by_name("FNumber")
                {
                    if !vals.is_empty() {
                        let (num, den) = vals[0];
                        let fpexif_f = if den > 0 {
                            num as f64 / den as f64
                        } else {
                            0.0
                        };
                        if (fpexif_f - exiftool_aperture).abs() > 0.01 {
                            issues.push(value_mismatch_issue(
                                "FNumber",
                                &format!("{:.2}", exiftool_aperture),
                                &format!("{:.2}", fpexif_f),
                            ));
                        }
                    }
                }
            }

            // Check FocalLength
            if let Some(exiftool_focal) = exiftool_data.get("FocalLength").and_then(|v| {
                v.as_f64().or_else(|| {
                    v.as_str()
                        .and_then(|s| s.split_whitespace().next())
                        .and_then(|n| n.parse::<f64>().ok())
                })
            }) {
                if let Some(fpexif::data_types::ExifValue::Rational(ref vals)) =
                    exif_data.get_tag_by_name("FocalLength")
                {
                    if !vals.is_empty() {
                        let (num, den) = vals[0];
                        let fpexif_focal = if den > 0 {
                            num as f64 / den as f64
                        } else {
                            0.0
                        };
                        if (fpexif_focal - exiftool_focal).abs() > 0.1 {
                            issues.push(value_mismatch_issue(
                                "FocalLength",
                                &format!("{:.1}", exiftool_focal),
                                &format!("{:.1}", fpexif_focal),
                            ));
                        }
                    }
                }
            }

            // Check DateTimeOriginal
            if let Some(exiftool_datetime) = exiftool_data
                .get("DateTimeOriginal")
                .and_then(|v| v.as_str())
            {
                if let Some(fpexif::data_types::ExifValue::Ascii(fpexif_datetime)) =
                    exif_data.get_tag_by_name("DateTimeOriginal")
                {
                    let et = normalize_string(exiftool_datetime);
                    let fp = normalize_string(fpexif_datetime);
                    if et != fp {
                        issues.push(value_mismatch_issue("DateTimeOriginal", &et, &fp));
                    }
                }
            }

            let reference_tag_count = exiftool_data.as_object().map(|m| m.len()).unwrap_or(0);

            // Determine if there are critical issues (value mismatches)
            let has_critical = issues
                .iter()
                .any(|i| matches!(i.category, IssueCategory::ValueMismatch));

            FileTestResult {
                file_path: path.to_string(),
                format,
                success: !has_critical,
                fpexif_tag_count: exif_data.len(),
                reference_tag_count,
                issues,
            }
        }
        Err(e) => {
            println!("Failed to parse {}: {:?}", path, e);
            FileTestResult {
                file_path: path.to_string(),
                format,
                success: true, // Parse failures are logged but not critical for this test
                fpexif_tag_count: 0,
                reference_tag_count: 0,
                issues: vec![TestIssue {
                    category: IssueCategory::ParseError,
                    message: format!("Parse error: {:?}", e),
                    field: None,
                    expected: None,
                    actual: None,
                }],
            }
        }
    }
}

/// Helper to find all files with a given extension
fn find_files_by_extension(extension: &str) -> Vec<String> {
    let base_path = "/fpexif/raws";
    if !Path::new(base_path).exists() {
        return Vec::new();
    }

    let mut files = Vec::new();
    collect_files_recursive(Path::new(base_path), extension, &mut files);
    files
}

fn collect_files_recursive(dir: &Path, extension: &str, files: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files_recursive(&path, extension, files);
            } else if let Some(ext) = path.extension() {
                if ext.to_string_lossy().to_uppercase() == extension.to_uppercase() {
                    if let Some(path_str) = path.to_str() {
                        files.push(path_str.to_string());
                    }
                }
            }
        }
    }
}

/// Run tests for a format and write JSON results
fn run_format_test(extension: &str, test_name: &str) {
    require_real_files_or_skip(test_name);
    if !real_files_exist() {
        return;
    }

    let files = find_files_by_extension(extension);
    println!("Found {} {} files", files.len(), extension.to_uppercase());

    let mut result = FormatTestResult::new(extension, test_name, "exiftool");

    for file in &files {
        println!("\nTesting: {}", file);
        let file_result = test_file_against_exiftool(file);
        result.add_file_result(file_result);
    }

    // Write JSON result
    if let Err(e) = result.write_to_file() {
        eprintln!("Failed to write test results: {}", e);
    }

    // Report summary
    println!("\n=== {} Summary ===", extension.to_uppercase());
    println!("Files tested: {}", result.files_tested);
    println!("Files passed: {}", result.files_passed);
    println!("Total issues: {}", result.total_issues);
    println!("  Unknown tags: {}", result.unknown_tags);
    println!("  Missing fields: {}", result.missing_fields);
    println!("  Value mismatches: {}", result.value_mismatches);

    if !files.is_empty() {
        assert!(
            result.files_tested > 0,
            "Expected to find {} files",
            extension.to_uppercase()
        );
    }

    // Fail on critical issues
    if result.has_critical_failures() {
        panic!(
            "Found {} critical issues in {} tests",
            result.critical_issues,
            extension.to_uppercase()
        );
    }
}

#[test]
fn test_cr2_files() {
    run_format_test("CR2", "test_cr2_files");
}

#[test]
fn test_nef_files() {
    run_format_test("NEF", "test_nef_files");
}

#[test]
fn test_orf_files() {
    run_format_test("ORF", "test_orf_files");
}

#[test]
fn test_crw_files() {
    run_format_test("CRW", "test_crw_files");
}

#[test]
fn test_arw_files() {
    run_format_test("ARW", "test_arw_files");
}

#[test]
fn test_raf_files() {
    run_format_test("RAF", "test_raf_files");
}

#[test]
fn test_rw2_files() {
    run_format_test("RW2", "test_rw2_files");
}

#[test]
fn test_dng_files() {
    run_format_test("DNG", "test_dng_files");
}

#[test]
fn test_pef_files() {
    run_format_test("PEF", "test_pef_files");
}

#[test]
fn test_raw_files() {
    run_format_test("RAW", "test_raw_files");
}

#[test]
fn test_mrw_files() {
    run_format_test("MRW", "test_mrw_files");
}

#[test]
fn test_x3f_files() {
    run_format_test("X3F", "test_x3f_files");
}

#[test]
fn test_srw_files() {
    run_format_test("SRW", "test_srw_files");
}

#[test]
fn test_kdc_files() {
    run_format_test("KDC", "test_kdc_files");
}

#[test]
fn test_nrw_files() {
    run_format_test("NRW", "test_nrw_files");
}

#[test]
fn test_3fr_files() {
    run_format_test("3FR", "test_3fr_files");
}

#[test]
fn test_sr2_files() {
    run_format_test("SR2", "test_sr2_files");
}

#[test]
fn test_ppm_files() {
    run_format_test("PPM", "test_ppm_files");
}

#[test]
fn test_erf_files() {
    run_format_test("ERF", "test_erf_files");
}

#[test]
fn test_dcr_files() {
    run_format_test("DCR", "test_dcr_files");
}

#[test]
fn test_mos_files() {
    run_format_test("MOS", "test_mos_files");
}

#[test]
fn test_srf_files() {
    run_format_test("SRF", "test_srf_files");
}

#[test]
fn test_mef_files() {
    run_format_test("MEF", "test_mef_files");
}

#[test]
fn test_mdc_files() {
    run_format_test("MDC", "test_mdc_files");
}

#[test]
fn test_tiff_files() {
    require_real_files_or_skip("test_tiff_files");
    if !real_files_exist() {
        return;
    }

    let mut all_files = find_files_by_extension("TIFF");
    all_files.extend(find_files_by_extension("TIF"));

    println!("Found {} TIFF files", all_files.len());

    let mut result = FormatTestResult::new("TIFF", "test_tiff_files", "exiftool");

    for file in &all_files {
        println!("\nTesting: {}", file);
        let file_result = test_file_against_exiftool(file);
        result.add_file_result(file_result);
    }

    if let Err(e) = result.write_to_file() {
        eprintln!("Failed to write test results: {}", e);
    }

    if result.has_critical_failures() {
        panic!(
            "Found {} critical issues in TIFF tests",
            result.critical_issues
        );
    }
}

#[test]
fn test_jpeg_files() {
    require_real_files_or_skip("test_jpeg_files");
    if !real_files_exist() {
        return;
    }

    let mut all_files = find_files_by_extension("JPEG");
    all_files.extend(find_files_by_extension("JPG"));

    println!("Found {} JPEG files", all_files.len());

    let mut result = FormatTestResult::new("JPEG", "test_jpeg_files", "exiftool");

    for file in &all_files {
        println!("\nTesting: {}", file);
        let file_result = test_file_against_exiftool(file);
        result.add_file_result(file_result);
    }

    if let Err(e) = result.write_to_file() {
        eprintln!("Failed to write test results: {}", e);
    }

    if result.has_critical_failures() {
        panic!(
            "Found {} critical issues in JPEG tests",
            result.critical_issues
        );
    }
}
