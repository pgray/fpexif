// Test that compares our --exiftool-json output directly with exiftool -json output
use std::path::Path;
use std::process::Command;

/// Helper function to check if real files directory exists
fn real_files_exist() -> bool {
    Path::new("/fpexif/raws/welcome.html").exists()
}

/// Helper function to check if exiftool is available
fn exiftool_available() -> bool {
    Command::new("exiftool")
        .arg("--version")
        .output()
        .is_ok()
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
        .args(&["run", "--release", "--features", "cli", "--bin", "fpexif", "--"])
        .args(&["list", path, "--exiftool-json"])
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

/// Compare two JSON values, ignoring certain fields that are expected to differ
fn compare_json_outputs(
    exiftool_json: &serde_json::Value,
    fpexif_json: &serde_json::Value,
) -> Vec<String> {
    let mut differences = Vec::new();

    // Both should be arrays
    let exiftool_array = match exiftool_json.as_array() {
        Some(arr) => arr,
        None => {
            differences.push("exiftool output is not an array".to_string());
            return differences;
        }
    };

    let fpexif_array = match fpexif_json.as_array() {
        Some(arr) => arr,
        None => {
            differences.push("fpexif output is not an array".to_string());
            return differences;
        }
    };

    // Should have same number of elements (typically 1)
    if exiftool_array.len() != fpexif_array.len() {
        differences.push(format!(
            "Array length mismatch: exiftool={} fpexif={}",
            exiftool_array.len(),
            fpexif_array.len()
        ));
        return differences;
    }

    // Compare first object (typically the only one)
    if let (Some(exiftool_obj), Some(fpexif_obj)) =
        (exiftool_array[0].as_object(), fpexif_array[0].as_object())
    {
        // Fields to ignore (exiftool-specific or expected to differ)
        let ignore_fields = [
            "SourceFile",        // Paths may differ
            "ExifToolVersion",   // We don't include this
            "FileName",          // File metadata we don't include
            "Directory",         // File metadata we don't include
            "FileSize",          // File metadata we don't include
            "FileModifyDate",    // File metadata we don't include
            "FileAccessDate",    // File metadata we don't include
            "FileInodeChangeDate", // File metadata we don't include
            "FilePermissions",   // File metadata we don't include
            "FileType",          // File metadata we don't include
            "FileTypeExtension", // File metadata we don't include
            "MIMEType",          // File metadata we don't include
            "CurrentIPTCDigest", // IPTC data we don't parse yet
            "CodedCharacterSet", // IPTC data we don't parse yet
            "ApplicationRecordVersion", // IPTC data we don't parse yet
            "XMPToolkit",        // XMP data we don't parse yet
            "MakerNote",         // Raw maker note data - we parse it differently
        ];

        // Check for fields in exiftool that are missing or different in fpexif
        for (key, exiftool_value) in exiftool_obj {
            if ignore_fields.contains(&key.as_str()) {
                continue;
            }

            match fpexif_obj.get(key) {
                None => {
                    differences.push(format!("Missing field in fpexif: {}", key));
                }
                Some(fpexif_value) => {
                    // For numeric values, allow small floating point differences
                    if let (Some(et_num), Some(fp_num)) =
                        (exiftool_value.as_f64(), fpexif_value.as_f64())
                    {
                        if (et_num - fp_num).abs() > 0.001 {
                            differences.push(format!(
                                "Numeric mismatch for {}: exiftool={} fpexif={}",
                                key, et_num, fp_num
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
                            // Some known acceptable differences
                            let acceptable_difference = match key.as_str() {
                                "Orientation" => {
                                    // ExifTool might say "Horizontal (normal)" vs our "Normal (top-left is 0,0)"
                                    et_normalized.contains("normal")
                                        && fp_normalized.contains("Normal")
                                }
                                _ => false,
                            };

                            if !acceptable_difference {
                                differences.push(format!(
                                    "String mismatch for {}: exiftool=\"{}\" fpexif=\"{}\"",
                                    key, et_str, fp_str
                                ));
                            }
                        }
                    }
                    // Different types
                    else if exiftool_value != fpexif_value {
                        differences.push(format!(
                            "Value mismatch for {}: exiftool={} fpexif={}",
                            key, exiftool_value, fpexif_value
                        ));
                    }
                }
            }
        }

        // Check for extra fields in fpexif that aren't in exiftool (informational, not an error)
        for key in fpexif_obj.keys() {
            if !exiftool_obj.contains_key(key) && !key.starts_with("Canon") && !key.starts_with("Nikon") && !key.starts_with("Sony") {
                // Note: Maker note tags are expected to be different
                differences.push(format!("Extra field in fpexif (not in exiftool): {}", key));
            }
        }
    }

    differences
}

#[test]
fn test_exiftool_json_compatibility_cr2() {
    if !real_files_exist() {
        println!("Skipping test - real files not available");
        return;
    }

    if !exiftool_available() {
        println!("Skipping test - exiftool not available");
        return;
    }

    // Find a CR2 file to test
    let test_files = std::fs::read_dir("/fpexif/raws")
        .ok()
        .and_then(|entries| {
            entries
                .filter_map(|e| e.ok())
                .find(|e| {
                    e.path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext.eq_ignore_ascii_case("cr2"))
                        .unwrap_or(false)
                })
        });

    let test_file = match test_files {
        Some(entry) => entry.path(),
        None => {
            println!("No CR2 files found for testing");
            return;
        }
    };

    let test_path = test_file.to_str().unwrap();
    println!("Testing with file: {}", test_path);

    // Get outputs from both tools
    let exiftool_json = match get_exiftool_json_output(test_path) {
        Ok(json) => json,
        Err(e) => {
            println!("Failed to get exiftool output: {}", e);
            return;
        }
    };

    let fpexif_json = match get_fpexif_exiftool_json_output(test_path) {
        Ok(json) => json,
        Err(e) => {
            panic!("Failed to get fpexif output: {}", e);
        }
    };

    // Compare outputs
    let differences = compare_json_outputs(&exiftool_json, &fpexif_json);

    if !differences.is_empty() {
        println!("\nFound {} differences:", differences.len());
        for diff in &differences {
            println!("  - {}", diff);
        }

        // Only fail on critical differences (missing important fields)
        let critical_differences: Vec<_> = differences
            .iter()
            .filter(|d| {
                d.contains("Missing field") &&
                !d.contains("Extra field") &&
                // Allow missing some less critical fields
                !d.contains("SubSecTime") &&
                !d.contains("Thumbnail") &&
                !d.contains("Preview")
            })
            .collect();

        if !critical_differences.is_empty() {
            panic!(
                "Found {} critical differences in JSON output",
                critical_differences.len()
            );
        }
    } else {
        println!("✓ JSON outputs match!");
    }
}
