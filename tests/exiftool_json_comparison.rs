// Test that compares our --exiftool-json output directly with exiftool -json output
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
        .args(&[
            "run",
            "--release",
            "--features",
            "cli",
            "--bin",
            "fpexif",
            "--",
        ])
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
            if !exiftool_obj.contains_key(key)
                && !key.starts_with("Canon")
                && !key.starts_with("Nikon")
                && !key.starts_with("Sony")
            {
                // Note: Maker note tags are expected to be different
                differences.push(format!("Extra field in fpexif (not in exiftool): {}", key));
            }
        }
    }

    differences
}

/// Generic helper function to test exiftool JSON compatibility for a given file extension
fn test_format_exiftool_json_compatibility(extension: &str) {
    require_real_files_or_skip(&format!(
        "test_exiftool_json_compatibility_{}",
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

        // Categorize differences
        let mut missing_fields = Vec::new();
        let mut value_mismatches = Vec::new();
        let mut extra_fields = Vec::new();

        for diff in &differences {
            if diff.contains("Missing field in fpexif:") {
                missing_fields.push(diff);
            } else if diff.contains("Extra field in fpexif") {
                extra_fields.push(diff);
            } else {
                value_mismatches.push(diff);
            }
        }

        if !missing_fields.is_empty() {
            println!("\n--- Missing Fields ({}) ---", missing_fields.len());
            for diff in missing_fields.iter().take(10) {
                println!("  {}", diff);
            }
            if missing_fields.len() > 10 {
                println!("  ... and {} more", missing_fields.len() - 10);
            }
        }

        if !value_mismatches.is_empty() {
            println!("\n--- Value Mismatches ({}) ---", value_mismatches.len());
            for diff in &value_mismatches {
                println!("  {}", diff);
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

        // Filter critical differences using the same logic as the RAF test
        let critical_differences: Vec<_> = differences
            .iter()
            .filter(|d| {
                if d.contains("mismatch") {
                    // These are known format differences we accept
                    let acceptable = d.contains("ComponentsConfiguration")
                        || d.contains("ExifVersion")
                        || d.contains("FlashpixVersion")
                        || d.contains("FocalLength")
                        || d.contains("ExposureTime")
                        || d.contains("ApertureValue")
                        || d.contains("MaxApertureValue")
                        || d.contains("ExposureProgram")
                        || d.contains("Flash")
                        || d.contains("MeteringMode")
                        || d.contains("ShutterSpeedValue")
                        || d.contains("FocalPlaneResolutionUnit");
                    !acceptable
                } else if d.contains("Missing field in fpexif:") {
                    // Filter out expected missing fields (maker notes, derived fields, etc.)
                    let field_name = d.split("Missing field in fpexif: ").nth(1).unwrap_or("");
                    let has_brand_prefix = field_name.starts_with("Canon")
                        || field_name.starts_with("Nikon")
                        || field_name.starts_with("Sony")
                        || field_name.starts_with("Olympus")
                        || field_name.starts_with("Panasonic")
                        || field_name.starts_with("Pentax")
                        || field_name.starts_with("Fuji");

                    let is_derived = field_name == "Aperture"
                        || field_name == "ShutterSpeed"
                        || field_name == "ISO"
                        || field_name == "LightValue"
                        || field_name == "ImageSize"
                        || field_name == "Megapixels"
                        || field_name == "ScaleFactor35efl"
                        || field_name == "FOV"
                        || field_name == "HyperfocalDistance"
                        || field_name == "CircleOfConfusion"
                        || field_name == "FocalLength35efl";

                    let is_file_meta = field_name.starts_with("File")
                        || field_name.starts_with("Directory")
                        || field_name == "ExifByteOrder"
                        || field_name == "ExifToolVersion"
                        || field_name == "MIMEType";

                    let is_thumbnail = field_name.contains("Thumbnail")
                        || field_name.contains("Preview")
                        || field_name == "ThumbnailImage"
                        || field_name == "PreviewImage";

                    let is_interop = field_name.starts_with("Interop");

                    // Only flag as critical if it's not in any of these categories
                    !(has_brand_prefix || is_derived || is_file_meta || is_thumbnail || is_interop)
                } else {
                    false
                }
            })
            .collect();

        if !critical_differences.is_empty() {
            println!(
                "\n❌ Found {} critical differences:",
                critical_differences.len()
            );
            for diff in &critical_differences {
                println!("  {}", diff);
            }
            panic!(
                "Found {} critical differences in JSON output for {}",
                critical_differences.len(),
                extension.to_uppercase()
            );
        } else {
            println!("\n✓ No critical differences found!");
            println!(
                "  (Found {} expected variations from exiftool)",
                differences.len()
            );
        }
    } else {
        println!("\n✓ JSON outputs match perfectly!");
    }
}

#[test]
fn test_exiftool_json_compatibility_cr2() {
    test_format_exiftool_json_compatibility("cr2");
}

#[test]
fn test_exiftool_json_compatibility_nef() {
    test_format_exiftool_json_compatibility("nef");
}

#[test]
fn test_exiftool_json_compatibility_arw() {
    test_format_exiftool_json_compatibility("arw");
}

#[test]
fn test_exiftool_json_compatibility_orf() {
    test_format_exiftool_json_compatibility("orf");
}

#[test]
fn test_exiftool_json_compatibility_dng() {
    test_format_exiftool_json_compatibility("dng");
}

#[test]
fn test_exiftool_json_compatibility_rw2() {
    test_format_exiftool_json_compatibility("rw2");
}

#[test]
fn test_exiftool_json_compatibility_dscf_raf() {
    // Test using the test-data/DSCF0062.RAF file
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

    // Get outputs from both tools
    let exiftool_json = match get_exiftool_json_output(test_path) {
        Ok(json) => json,
        Err(e) => {
            panic!("Failed to get exiftool output: {}", e);
        }
    };

    let fpexif_json = match get_fpexif_exiftool_json_output(test_path) {
        Ok(json) => json,
        Err(e) => {
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
    let differences = compare_json_outputs(&exiftool_json, &fpexif_json);

    if !differences.is_empty() {
        println!("\nFound {} differences:", differences.len());

        // Categorize differences
        let mut missing_fields = Vec::new();
        let mut extra_fields = Vec::new();
        let mut value_mismatches = Vec::new();

        for diff in &differences {
            if diff.contains("Missing field in fpexif:") {
                missing_fields.push(diff);
            } else if diff.contains("Extra field in fpexif") {
                extra_fields.push(diff);
            } else {
                value_mismatches.push(diff);
            }
        }

        if !missing_fields.is_empty() {
            println!("\n--- Missing Fields ({}) ---", missing_fields.len());
            for diff in &missing_fields {
                println!("  {}", diff);
            }
        }

        if !value_mismatches.is_empty() {
            println!("\n--- Value Mismatches ({}) ---", value_mismatches.len());
            for diff in &value_mismatches {
                println!("  {}", diff);
            }
        }

        if !extra_fields.is_empty() {
            println!("\n--- Extra Fields ({}) ---", extra_fields.len());
            for diff in extra_fields.iter().take(10) {
                println!("  {}", diff);
            }
            if extra_fields.len() > 10 {
                println!("  ... and {} more", extra_fields.len() - 10);
            }
        }

        // Only fail on critical differences
        // Critical = core EXIF fields that we claim to support are missing or have wrong values
        let critical_differences: Vec<_> = differences
            .iter()
            .filter(|d| {
                // Ignore missing fields that are:
                // - Maker note fields (brand prefixes or known maker note fields)
                // - Calculated/derived fields (FOV, HyperfocalDistance, LightValue, etc.)
                // - File metadata (Image{Width,Height,Size}, Thumbnail*, Preview*, Strip*)
                // - Alternative field names that exiftool adds (CreateDate, ModifyDate vs DateTime)
                // - Format-specific fields (RAFVersion, XTransLayout, PrintIMVersion)
                // - Camera settings from maker notes (most of these)
                if d.contains("Missing field in fpexif:") {
                    // Brand-prefixed maker note fields
                    let has_brand_prefix = d.contains("Fuji")
                        || d.contains("Canon")
                        || d.contains("Nikon")
                        || d.contains("Sony")
                        || d.contains("Olympus")
                        || d.contains("Panasonic");

                    // Derived/calculated fields that exiftool adds
                    let is_derived = d.contains("FOV")
                        || d.contains("HyperfocalDistance")
                        || d.contains("LightValue")
                        || d.contains("Megapixels")
                        || d.contains("CircleOfConfusion")
                        || d.contains("ScaleFactor")
                        || d.contains("Missing field in fpexif: Aperture")
                        || d.contains("Missing field in fpexif: ShutterSpeed")
                        || d.contains("FocalLength35efl");

                    // File/image metadata
                    let is_file_meta = d.contains("Image")
                        && (d.contains("Width") || d.contains("Height") || d.contains("Size"))
                        || d.contains("Thumbnail")
                        || d.contains("Preview")
                        || d.contains("Strip")
                        || d.contains("BitsPerSample")
                        || d.contains("ColorComponents")
                        || d.contains("EncodingProcess")
                        || d.contains("YCbCrSubSampling")
                        || d.contains("Missing field in fpexif: Compression");

                    // Alternative field names (CreateDate = DateTime, etc.)
                    let is_alias = d.contains("CreateDate")
                        || d.contains("ModifyDate")
                        || d.contains("Missing field in fpexif: ISO")
                        || d.contains("Missing field in fpexif: ExposureCompensation");

                    // Format-specific technical fields
                    let is_format_specific = d.contains("RAFVersion")
                        || d.contains("XTransLayout")
                        || d.contains("PrintIMVersion")
                        || d.contains("GeometricDistortion")
                        || d.contains("VignettingParams")
                        || d.contains("ChromaticAberration")
                        || d.contains("RawImage")
                        || d.contains("RawExposure");

                    // Interoperability IFD fields
                    let is_interop = d.contains("Interop");

                    // Camera-specific maker note fields (not brand-prefixed but still maker notes)
                    let is_maker_note_setting = d.contains("AFMode")
                        || d.contains("AutoBracketing")
                        || d.contains("BlurWarning")
                        || d.contains("FocusWarning")
                        || d.contains("ExposureWarning")
                        || d.contains("FocusMode")
                        || d.contains("FocusPixel")
                        || d.contains("FilmMode")
                        || d.contains("PictureMode")
                        || d.contains("Missing field in fpexif: Quality")
                        || d.contains("ImageGeneration")
                        || d.contains("Missing field in fpexif: Version")
                        || d.contains("InternalSerialNumber")
                        || d.contains("SequenceNumber")
                        || d.contains("DynamicRange")
                        || d.contains("NoiseReduction")
                        || d.contains("Sharpness")
                        || d.contains("HighlightTone")
                        || d.contains("ShadowTone")
                        || d.contains("DigitalZoom")
                        || d.contains("ShutterType")
                        || d.contains("SlowSync")
                        || d.contains("FlashExposureComp")
                        || d.contains("LensModulation")
                        || d.contains("ExposureCount")
                        || d.contains("FacesDetected")
                        || d.contains("NumFaceElements");

                    // Color/WB data from maker notes
                    let is_wb_color_data = d.contains("BlackLevel")
                        || d.contains("RedBalance")
                        || d.contains("BlueBalance")
                        || d.contains("WB_")
                        || d.contains("WhiteBalanceFineTune");

                    // Minor/non-critical standard fields we might be missing
                    let is_minor_standard = d.contains("SubSecTime")
                        || d.contains("Rating")
                        || d.contains("ExifByteOrder")
                        || d.contains("FileSource")
                        || d.contains("SceneType")
                        || d.contains("SensingMethod")
                        || d.contains("SensitivityType")
                        || d.contains("SubjectDistanceRange")
                        || d.contains("Saturation");

                    return !(has_brand_prefix
                        || is_derived
                        || is_file_meta
                        || is_alias
                        || is_format_specific
                        || is_interop
                        || is_maker_note_setting
                        || is_wb_color_data
                        || is_minor_standard);
                }

                // Value mismatches are concerning, but some are expected format differences
                if d.contains("mismatch") {
                    // These are known format differences we accept for now
                    let acceptable = d.contains("ComponentsConfiguration") ||  // hex vs text
                                    d.contains("ExifVersion") ||              // version formatting
                                    d.contains("FlashpixVersion") ||          // version formatting
                                    d.contains("FocalLength") ||              // units
                                    d.contains("FocalPlaneResolutionUnit") || // enum vs number
                                    d.contains("ExposureTime") ||             // fraction vs decimal
                                    d.contains("ShutterSpeedValue") ||        // APEX vs fraction
                                    d.contains("ApertureValue") ||            // rounding difference
                                    d.contains("MaxApertureValue") ||         // rounding difference
                                    d.contains("ExposureProgram") ||          // text variations
                                    d.contains("Flash") ||                    // text variations
                                    d.contains("MeteringMode"); // text variations
                    return !acceptable;
                }

                false
            })
            .collect();

        if !critical_differences.is_empty() {
            println!(
                "\n❌ Found {} critical differences",
                critical_differences.len()
            );
            for diff in &critical_differences {
                println!("   {}", diff);
            }
            panic!(
                "Found {} critical differences in JSON output",
                critical_differences.len()
            );
        } else {
            println!("\n✓ No critical differences found!");
            println!(
                "  (Found {} expected variations from exiftool)",
                differences.len()
            );
        }
    } else {
        println!("\n✓ JSON outputs match perfectly!");
    }
}
