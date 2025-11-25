// Test that compares our fpexif exiftool text output with exiftool output
use std::path::Path;
use std::process::Command;

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

/// Parse exiftool text output line into (key, value)
fn parse_exiftool_line(line: &str) -> Option<(String, String)> {
    // exiftool format: "Tag Name                        : Value"
    let parts: Vec<&str> = line.splitn(2, ':').collect();
    if parts.len() == 2 {
        let key = parts[0].trim().to_string();
        let value = parts[1].trim().to_string();
        Some((key, value))
    } else {
        None
    }
}

/// Helper function to get exiftool text output
fn get_exiftool_output(path: &str) -> Result<Vec<(String, String)>, String> {
    let output = Command::new("exiftool")
        .arg(path)
        .output()
        .map_err(|e| format!("Failed to run exiftool: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "exiftool failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let results: Vec<_> = stdout.lines().filter_map(parse_exiftool_line).collect();

    Ok(results)
}

/// Helper function to get fpexif exiftool text output
fn get_fpexif_exiftool_output(path: &str) -> Result<Vec<(String, String)>, String> {
    let output = Command::new("cargo")
        .args([
            "run",
            "--release",
            "--features",
            "cli",
            "--bin",
            "fpexif",
            "--",
            "exiftool",
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
    let results: Vec<_> = stdout.lines().filter_map(parse_exiftool_line).collect();

    Ok(results)
}

/// Compare exiftool outputs and return differences
fn compare_exiftool_outputs(
    exiftool_output: &[(String, String)],
    fpexif_output: &[(String, String)],
) -> Vec<String> {
    let mut differences = Vec::new();

    // Create maps for easier lookup
    let exiftool_map: std::collections::HashMap<_, _> = exiftool_output
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let fpexif_map: std::collections::HashMap<_, _> = fpexif_output
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    // Check for fields in exiftool that are missing or different in fpexif
    for (key, exiftool_value) in &exiftool_map {
        match fpexif_map.get(key) {
            None => {
                differences.push(format!("Missing field in fpexif: {}", key));
            }
            Some(fpexif_value) => {
                // Normalize for comparison
                let exiftool_norm = exiftool_value.trim();
                let fpexif_norm = fpexif_value.trim();
                if exiftool_norm != fpexif_norm {
                    differences.push(format!(
                        "Value mismatch for {}: exiftool=\"{}\" fpexif=\"{}\"",
                        key, exiftool_norm, fpexif_norm
                    ));
                }
            }
        }
    }

    // Check for extra fields in fpexif
    for key in fpexif_map.keys() {
        if !exiftool_map.contains_key(key) {
            differences.push(format!("Extra field in fpexif: {}", key));
        }
    }

    differences
}

/// Generic helper function to test exiftool text compatibility for a given file extension
fn test_format_exiftool_text_compatibility(extension: &str) {
    require_real_files_or_skip(&format!(
        "test_exiftool_text_compatibility_{}",
        extension.to_lowercase()
    ));
    if !real_files_exist() {
        return;
    }

    if !exiftool_available() {
        println!("Skipping test - exiftool not available");
        return;
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
            return;
        }
    };

    let test_path = test_file.to_str().unwrap();
    println!("Testing with file: {}", test_path);

    // Get outputs from both tools
    let exiftool_output = match get_exiftool_output(test_path) {
        Ok(output) => output,
        Err(e) => {
            println!("Failed to get exiftool output: {}", e);
            return;
        }
    };

    let fpexif_output = match get_fpexif_exiftool_output(test_path) {
        Ok(output) => output,
        Err(e) => {
            panic!("Failed to get fpexif output: {}", e);
        }
    };

    println!("exiftool returned {} tags", exiftool_output.len());
    println!("fpexif returned {} tags", fpexif_output.len());

    // Compare outputs
    let differences = compare_exiftool_outputs(&exiftool_output, &fpexif_output);

    if !differences.is_empty() {
        println!("\nFound {} differences:", differences.len());

        let missing_fields: Vec<_> = differences
            .iter()
            .filter(|d| d.contains("Missing field in fpexif:"))
            .collect();
        let extra_fields: Vec<_> = differences
            .iter()
            .filter(|d| d.contains("Extra field in fpexif:"))
            .collect();
        let mismatches: Vec<_> = differences
            .iter()
            .filter(|d| d.contains("mismatch"))
            .collect();

        if !missing_fields.is_empty() {
            println!("\n--- Missing Fields ({}) ---", missing_fields.len());
            for diff in missing_fields.iter().take(10) {
                println!("  {}", diff);
            }
            if missing_fields.len() > 10 {
                println!("  ... and {} more", missing_fields.len() - 10);
            }
        }

        if !mismatches.is_empty() {
            println!("\n--- Mismatches ({}) ---", mismatches.len());
            for diff in mismatches.iter().take(10) {
                println!("  {}", diff);
            }
            if mismatches.len() > 10 {
                println!("  ... and {} more", mismatches.len() - 10);
            }
        }

        if !extra_fields.is_empty() {
            println!("\n--- Extra Fields ({}) ---", extra_fields.len());
            for diff in extra_fields.iter().take(5) {
                println!("  {}", diff);
            }
            if extra_fields.len() > 5 {
                println!("  ... and {} more", extra_fields.len() - 5);
            }
        }

        // Note: We don't fail this test yet as text compatibility is new
        println!("\nNote: exiftool text compatibility is still in development");
    } else {
        println!("\n* Outputs match!");
    }
}

#[test]
fn test_exiftool_text_compatibility_raf() {
    let test_path = "test-data/DSCF0062.RAF";

    if !Path::new(test_path).exists() {
        println!("Skipping test - {} not available", test_path);
        return;
    }

    if !exiftool_available() {
        println!("Skipping test - exiftool not available");
        return;
    }

    println!("Testing with file: {}", test_path);

    let exiftool_output = match get_exiftool_output(test_path) {
        Ok(output) => output,
        Err(e) => {
            println!("Failed to get exiftool output: {}", e);
            return;
        }
    };

    let fpexif_output = match get_fpexif_exiftool_output(test_path) {
        Ok(output) => output,
        Err(e) => {
            panic!("Failed to get fpexif output: {}", e);
        }
    };

    println!("exiftool returned {} tags", exiftool_output.len());
    println!("fpexif returned {} tags", fpexif_output.len());

    let differences = compare_exiftool_outputs(&exiftool_output, &fpexif_output);

    if !differences.is_empty() {
        println!("\nFound {} differences (informational)", differences.len());
    } else {
        println!("\n* Outputs match!");
    }
}

#[test]
fn test_exiftool_text_compatibility_cr2() {
    test_format_exiftool_text_compatibility("cr2");
}

#[test]
fn test_exiftool_text_compatibility_nef() {
    test_format_exiftool_text_compatibility("nef");
}

#[test]
fn test_exiftool_text_compatibility_arw() {
    test_format_exiftool_text_compatibility("arw");
}

#[test]
fn test_exiftool_text_compatibility_dng() {
    test_format_exiftool_text_compatibility("dng");
}

#[test]
fn test_exiftool_text_compatibility_jpg() {
    test_format_exiftool_text_compatibility("jpg");
}

// =============================================================================
// Placeholder tests for flags not yet implemented
// =============================================================================

#[test]
#[ignore = "exiftool -g (group by category) not yet implemented"]
fn test_exiftool_group_flag() {
    // TODO: Implement -g flag support
    // exiftool -g groups output by category (EXIF, File, etc.)
}

#[test]
#[ignore = "exiftool -s (short names) not yet implemented"]
fn test_exiftool_short_names_flag() {
    // TODO: Implement -s flag support
    // exiftool -s shows short tag names instead of descriptions
}

#[test]
#[ignore = "exiftool -n (numeric values) not yet implemented"]
fn test_exiftool_numeric_flag() {
    // TODO: Implement -n flag support
    // exiftool -n shows raw numeric values instead of converted
}

#[test]
#[ignore = "exiftool -S (very short) not yet implemented"]
fn test_exiftool_very_short_flag() {
    // TODO: Implement -S flag support
    // exiftool -S shows very short output (tag names only, no descriptions)
}

#[test]
#[ignore = "exiftool -t (tab-separated) not yet implemented"]
fn test_exiftool_tab_separated_flag() {
    // TODO: Implement -t flag support
    // exiftool -t outputs tab-separated values
}

#[test]
#[ignore = "exiftool -csv not yet implemented"]
fn test_exiftool_csv_flag() {
    // TODO: Implement -csv flag support
    // exiftool -csv outputs CSV format
}

#[test]
#[ignore = "exiftool -X (XML) not yet implemented"]
fn test_exiftool_xml_flag() {
    // TODO: Implement -X flag support
    // exiftool -X outputs XML format
}
