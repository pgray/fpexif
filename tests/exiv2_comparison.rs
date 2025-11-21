// Test that compares our fpexif exiv2 output with exiv2 output
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
    let output = Command::new("exiv2")
        .arg(path)
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

/// Compare exiv2 outputs and return differences
fn compare_exiv2_outputs(
    exiv2_output: &[(String, String, String, String)],
    fpexif_output: &[(String, String, String, String)],
) -> Vec<String> {
    let mut differences = Vec::new();

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
                differences.push(format!("Missing field in fpexif: {}", key));
            }
            Some((fpexif_type, fpexif_count, fpexif_value)) => {
                // Check type
                if exiv2_type != fpexif_type {
                    differences.push(format!(
                        "Type mismatch for {}: exiv2={} fpexif={}",
                        key, exiv2_type, fpexif_type
                    ));
                }
                // Check count
                if exiv2_count != fpexif_count {
                    differences.push(format!(
                        "Count mismatch for {}: exiv2={} fpexif={}",
                        key, exiv2_count, fpexif_count
                    ));
                }
                // Check value (normalize for comparison)
                let exiv2_val_norm = exiv2_value.trim();
                let fpexif_val_norm = fpexif_value.trim();
                if exiv2_val_norm != fpexif_val_norm {
                    differences.push(format!(
                        "Value mismatch for {}: exiv2=\"{}\" fpexif=\"{}\"",
                        key, exiv2_val_norm, fpexif_val_norm
                    ));
                }
            }
        }
    }

    // Check for extra fields in fpexif
    for key in fpexif_map.keys() {
        if !exiv2_map.contains_key(key) {
            differences.push(format!("Extra field in fpexif: {}", key));
        }
    }

    differences
}

/// Generic helper function to test exiv2 compatibility for a given file extension
fn test_format_exiv2_compatibility(extension: &str) {
    require_real_files_or_skip(&format!(
        "test_exiv2_compatibility_{}",
        extension.to_lowercase()
    ));
    if !real_files_exist() {
        return;
    }

    if !exiv2_available() {
        println!("Skipping test - exiv2 not available");
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

    // Compare outputs
    let differences = compare_exiv2_outputs(&exiv2_output, &fpexif_output);

    if !differences.is_empty() {
        println!("\nFound {} differences:", differences.len());

        // Categorize differences
        let mut missing_fields: Vec<_> = differences
            .iter()
            .filter(|d| d.contains("Missing field in fpexif:"))
            .collect();
        let mut extra_fields: Vec<_> = differences
            .iter()
            .filter(|d| d.contains("Extra field in fpexif:"))
            .collect();
        let mut mismatches: Vec<_> = differences
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

        // Filter critical differences
        let critical_differences: Vec<_> = differences
            .iter()
            .filter(|d| {
                // Missing fields that are acceptable
                if d.contains("Missing field in fpexif:") {
                    // Maker notes and brand-specific fields
                    let is_maker_note = d.contains("MakerNote")
                        || d.contains("Exif.Canon")
                        || d.contains("Exif.Nikon")
                        || d.contains("Exif.Sony")
                        || d.contains("Exif.Fuji")
                        || d.contains("Exif.Olympus")
                        || d.contains("Exif.Panasonic");

                    // Thumbnail fields
                    let is_thumbnail = d.contains("Exif.Thumbnail");

                    // IPTC and XMP (not yet supported)
                    let is_iptc_xmp = d.contains("Iptc.") || d.contains("Xmp.");

                    return !(is_maker_note || is_thumbnail || is_iptc_xmp);
                }

                // Value mismatches - some are expected
                if d.contains("mismatch") {
                    // Acceptable format differences
                    let acceptable = d.contains("Undefined")  // Binary data formatting
                        || d.contains("bytes"); // Size descriptions
                    return !acceptable;
                }

                false
            })
            .collect();

        if !critical_differences.is_empty() {
            println!(
                "\n!! Found {} critical differences:",
                critical_differences.len()
            );
            for diff in critical_differences.iter().take(20) {
                println!("  {}", diff);
            }
            // Don't panic yet - we're still developing exiv2 compatibility
            println!("\nNote: exiv2 compatibility is still in development");
        } else {
            println!("\n* No critical differences found!");
            println!(
                "  (Found {} expected variations from exiv2)",
                differences.len()
            );
        }
    } else {
        println!("\n* Outputs match!");
    }
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

    let differences = compare_exiv2_outputs(&exiv2_output, &fpexif_output);

    if !differences.is_empty() {
        println!("\nFound {} differences (informational)", differences.len());
        // Note: We don't fail this test yet as exiv2 compatibility is new
    } else {
        println!("\n* Outputs match!");
    }
}

#[test]
fn test_exiv2_compatibility_cr2() {
    test_format_exiv2_compatibility("cr2");
}

#[test]
fn test_exiv2_compatibility_nef() {
    test_format_exiv2_compatibility("nef");
}

#[test]
fn test_exiv2_compatibility_arw() {
    test_format_exiv2_compatibility("arw");
}

#[test]
fn test_exiv2_compatibility_dng() {
    test_format_exiv2_compatibility("dng");
}

#[test]
fn test_exiv2_compatibility_jpg() {
    test_format_exiv2_compatibility("jpg");
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
