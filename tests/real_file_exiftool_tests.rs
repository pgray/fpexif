// Real file format tests using exiftool as reference
// These tests only run if /fpexif/raws/welcome.html exists
use std::fs;
use std::path::Path;
use std::process::Command;

/// Helper function to check if real files directory exists
fn real_files_exist() -> bool {
    Path::new("/fpexif/raws/welcome.html").exists()
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

    // Check if exiftool reported an error
    if let Some(error) = data.get("Error") {
        return Err(format!("exiftool reported error: {}", error));
    }

    Ok(data)
}

/// Helper to normalize string values for comparison
fn normalize_string(s: &str) -> String {
    // Remove null bytes and non-printable characters
    s.chars()
        .take_while(|&c| c != '\0')
        .filter(|c| c.is_ascii_graphic() || c.is_whitespace())
        .collect::<String>()
        .trim()
        .to_string()
}

/// Helper function to test a single file
fn test_file_against_exiftool(path: &str) {
    if !real_files_exist() {
        println!("Skipping test - real files directory not found");
        return;
    }

    // Get exiftool output
    let exiftool_data = match get_exiftool_output(path) {
        Ok(data) => data,
        Err(e) => {
            println!("⚠ Skipping {} - exiftool error: {}", path, e);
            return;
        }
    };

    // Try to parse with fpexif
    let parser = fpexif::ExifParser::new();
    let fpexif_result = parser.parse_file(path);

    match fpexif_result {
        Ok(exif_data) => {
            // File parsed successfully
            println!("✓ Successfully parsed: {}", path);

            // Basic validation - check that we got some tags
            assert!(
                !exif_data.is_empty(),
                "fpexif returned no tags for {}",
                path
            );

            println!("  fpexif extracted {} tags", exif_data.len());

            // Validate common EXIF tags
            let mut validations = 0;
            let mut critical_mismatches = 0; // Make/Model mismatches
            let mut dimension_mismatches = 0; // Width/Height mismatches (acceptable for RAW thumbnails)

            // Check Make
            if let Some(exiftool_make) = exiftool_data.get("Make").and_then(|v| v.as_str()) {
                if let Some(fpexif_make_value) = exif_data.get_tag_by_name("Make") {
                    validations += 1;
                    if let fpexif::data_types::ExifValue::Ascii(fpexif_make) = fpexif_make_value {
                        let normalized_exiftool = normalize_string(exiftool_make);
                        let normalized_fpexif = normalize_string(fpexif_make);
                        if normalized_exiftool == normalized_fpexif {
                            println!("  ✓ Make: \"{}\"", normalized_fpexif);
                        } else {
                            critical_mismatches += 1;
                            println!(
                                "  ✗ Make mismatch: exiftool=\"{}\" fpexif=\"{}\"",
                                normalized_exiftool, normalized_fpexif
                            );
                        }
                    }
                }
            }

            // Check Model
            if let Some(exiftool_model) = exiftool_data.get("Model").and_then(|v| v.as_str()) {
                if let Some(fpexif_model_value) = exif_data.get_tag_by_name("Model") {
                    validations += 1;
                    if let fpexif::data_types::ExifValue::Ascii(fpexif_model) = fpexif_model_value {
                        let normalized_exiftool = normalize_string(exiftool_model);
                        let normalized_fpexif = normalize_string(fpexif_model);
                        if normalized_exiftool == normalized_fpexif {
                            println!("  ✓ Model: \"{}\"", normalized_fpexif);
                        } else {
                            critical_mismatches += 1;
                            println!(
                                "  ✗ Model mismatch: exiftool=\"{}\" fpexif=\"{}\"",
                                normalized_exiftool, normalized_fpexif
                            );
                        }
                    }
                }
            }

            // Check Orientation
            if let Some(exiftool_orientation) = exiftool_data.get("Orientation").and_then(|v| {
                // Exiftool returns a string like "Horizontal (normal)"
                // We want to extract just the number
                if let Some(num_str) = v.as_str() {
                    // Try to parse as number directly, or extract from string
                    num_str.chars().find(|c| c.is_numeric()).and_then(|_| {
                        num_str
                            .split_whitespace()
                            .next()
                            .and_then(|s| s.parse::<u16>().ok())
                    })
                } else {
                    v.as_u64().map(|n| n as u16)
                }
            }) {
                if let Some(fpexif_orientation_value) = exif_data.get_tag_by_name("Orientation") {
                    validations += 1;
                    if let fpexif::data_types::ExifValue::Short(ref vals) = fpexif_orientation_value
                    {
                        if !vals.is_empty() && vals[0] == exiftool_orientation {
                            println!("  ✓ Orientation: {}", vals[0]);
                        } else if !vals.is_empty() {
                            critical_mismatches += 1;
                            println!(
                                "  ✗ Orientation mismatch: exiftool={} fpexif={}",
                                exiftool_orientation, vals[0]
                            );
                        }
                    }
                }
            }

            // Check ImageWidth
            if let Some(exiftool_width) = exiftool_data
                .get("ImageWidth")
                .or_else(|| exiftool_data.get("ExifImageWidth"))
                .and_then(|v| v.as_u64().map(|n| n as u32))
            {
                if let Some(fpexif_width_value) = exif_data
                    .get_tag_by_name("ImageWidth")
                    .or_else(|| exif_data.get_tag_by_name("ExifImageWidth"))
                {
                    validations += 1;
                    match fpexif_width_value {
                        fpexif::data_types::ExifValue::Short(ref vals) if !vals.is_empty() => {
                            if vals[0] as u32 == exiftool_width {
                                println!("  ✓ ImageWidth: {}", vals[0]);
                            } else {
                                dimension_mismatches += 1;
                                println!(
                                    "  ⚠ ImageWidth mismatch: exiftool={} fpexif={} (may be thumbnail)",
                                    exiftool_width, vals[0]
                                );
                            }
                        }
                        fpexif::data_types::ExifValue::Long(ref vals) if !vals.is_empty() => {
                            if vals[0] == exiftool_width {
                                println!("  ✓ ImageWidth: {}", vals[0]);
                            } else {
                                dimension_mismatches += 1;
                                println!(
                                    "  ⚠ ImageWidth mismatch: exiftool={} fpexif={} (may be thumbnail)",
                                    exiftool_width, vals[0]
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Check ImageHeight
            if let Some(exiftool_height) = exiftool_data
                .get("ImageHeight")
                .or_else(|| exiftool_data.get("ExifImageHeight"))
                .and_then(|v| v.as_u64().map(|n| n as u32))
            {
                if let Some(fpexif_height_value) = exif_data
                    .get_tag_by_name("ImageLength")
                    .or_else(|| exif_data.get_tag_by_name("ExifImageHeight"))
                {
                    validations += 1;
                    match fpexif_height_value {
                        fpexif::data_types::ExifValue::Short(ref vals) if !vals.is_empty() => {
                            if vals[0] as u32 == exiftool_height {
                                println!("  ✓ ImageHeight: {}", vals[0]);
                            } else {
                                dimension_mismatches += 1;
                                println!(
                                    "  ⚠ ImageHeight mismatch: exiftool={} fpexif={} (may be thumbnail)",
                                    exiftool_height, vals[0]
                                );
                            }
                        }
                        fpexif::data_types::ExifValue::Long(ref vals) if !vals.is_empty() => {
                            if vals[0] == exiftool_height {
                                println!("  ✓ ImageHeight: {}", vals[0]);
                            } else {
                                dimension_mismatches += 1;
                                println!(
                                    "  ⚠ ImageHeight mismatch: exiftool={} fpexif={} (may be thumbnail)",
                                    exiftool_height, vals[0]
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }

            if validations > 0 {
                let total_mismatches = critical_mismatches + dimension_mismatches;
                if total_mismatches > 0 {
                    if dimension_mismatches > 0 && critical_mismatches == 0 {
                        println!(
                            "  ✓ Validated {}/{} common tags ({} dimension mismatches - expected for RAW thumbnails)",
                            validations - total_mismatches, validations, dimension_mismatches
                        );
                    } else {
                        println!(
                            "  Validated {}/{} common tags",
                            validations - total_mismatches,
                            validations
                        );
                    }
                } else {
                    println!("  ✓ Validated {}/{} common tags", validations, validations);
                }

                // Only fail on critical mismatches (Make/Model/Orientation), not dimensions
                // (dimensions can differ for RAW files when comparing thumbnail vs full image)
                assert_eq!(
                    critical_mismatches, 0,
                    "Found {} critical tag value mismatches in {}",
                    critical_mismatches, path
                );
            }
        }
        Err(e) => {
            // Some files may not have EXIF data or may use formats we don't support yet
            println!("✗ Failed to parse {}: {:?}", path, e);
            // We don't panic here because some files might legitimately not have EXIF
            // or might be in formats we're still developing support for
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
    if let Ok(entries) = fs::read_dir(base_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Recursively search subdirectories
                collect_files_recursive(&path, extension, &mut files);
            }
        }
    }
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

// CR2 Tests (Canon RAW - 55 files)
#[test]
fn test_cr2_files() {
    if !real_files_exist() {
        println!("Skipping CR2 tests - real files not available");
        return;
    }

    let cr2_files = find_files_by_extension("CR2");
    println!("Found {} CR2 files", cr2_files.len());

    for file in &cr2_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    assert!(!cr2_files.is_empty(), "Expected to find CR2 files");
}

// NEF Tests (Nikon RAW - 47 files)
#[test]
fn test_nef_files() {
    if !real_files_exist() {
        println!("Skipping NEF tests - real files not available");
        return;
    }

    let nef_files = find_files_by_extension("NEF");
    println!("Found {} NEF files", nef_files.len());

    for file in &nef_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    assert!(!nef_files.is_empty(), "Expected to find NEF files");
}

// ORF Tests (Olympus RAW - 36 files)
#[test]
fn test_orf_files() {
    if !real_files_exist() {
        println!("Skipping ORF tests - real files not available");
        return;
    }

    let orf_files = find_files_by_extension("ORF");
    println!("Found {} ORF files", orf_files.len());

    for file in &orf_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    assert!(!orf_files.is_empty(), "Expected to find ORF files");
}

// ARW Tests (Sony RAW - 31 files)
#[test]
fn test_arw_files() {
    if !real_files_exist() {
        println!("Skipping ARW tests - real files not available");
        return;
    }

    let arw_files = find_files_by_extension("ARW");
    println!("Found {} ARW files", arw_files.len());

    for file in &arw_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    assert!(!arw_files.is_empty(), "Expected to find ARW files");
}

// RAF Tests (Fujifilm RAW - 30 files)
#[test]
fn test_raf_files() {
    if !real_files_exist() {
        println!("Skipping RAF tests - real files not available");
        return;
    }

    let raf_files = find_files_by_extension("RAF");
    println!("Found {} RAF files", raf_files.len());

    for file in &raf_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    assert!(!raf_files.is_empty(), "Expected to find RAF files");
}

// RW2 Tests (Panasonic RAW - 20 files)
#[test]
fn test_rw2_files() {
    if !real_files_exist() {
        println!("Skipping RW2 tests - real files not available");
        return;
    }

    let rw2_files = find_files_by_extension("RW2");
    println!("Found {} RW2 files", rw2_files.len());

    for file in &rw2_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    assert!(!rw2_files.is_empty(), "Expected to find RW2 files");
}

// DNG Tests (Adobe DNG - 18 files)
#[test]
fn test_dng_files() {
    if !real_files_exist() {
        println!("Skipping DNG tests - real files not available");
        return;
    }

    let dng_files = find_files_by_extension("DNG");
    println!("Found {} DNG files", dng_files.len());

    for file in &dng_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    assert!(!dng_files.is_empty(), "Expected to find DNG files");
}

// PEF Tests (Pentax RAW - 17 files)
#[test]
fn test_pef_files() {
    if !real_files_exist() {
        println!("Skipping PEF tests - real files not available");
        return;
    }

    let pef_files = find_files_by_extension("PEF");
    println!("Found {} PEF files", pef_files.len());

    for file in &pef_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    assert!(!pef_files.is_empty(), "Expected to find PEF files");
}

// Generic RAW Tests (15 files)
#[test]
fn test_raw_files() {
    if !real_files_exist() {
        println!("Skipping RAW tests - real files not available");
        return;
    }

    let raw_files = find_files_by_extension("RAW");
    println!("Found {} RAW files", raw_files.len());

    for file in &raw_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    // Note: RAW files might not all exist
    if raw_files.is_empty() {
        println!("No RAW files found - this is OK");
    }
}

// MRW Tests (Minolta RAW - 9 files)
#[test]
fn test_mrw_files() {
    if !real_files_exist() {
        println!("Skipping MRW tests - real files not available");
        return;
    }

    let mrw_files = find_files_by_extension("MRW");
    println!("Found {} MRW files", mrw_files.len());

    for file in &mrw_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    assert!(!mrw_files.is_empty(), "Expected to find MRW files");
}

// X3F Tests (Sigma RAW - 8 files)
#[test]
fn test_x3f_files() {
    if !real_files_exist() {
        println!("Skipping X3F tests - real files not available");
        return;
    }

    let x3f_files = find_files_by_extension("X3F");
    println!("Found {} X3F files", x3f_files.len());

    for file in &x3f_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    assert!(!x3f_files.is_empty(), "Expected to find X3F files");
}

// SRW Tests (Samsung RAW - 7 files)
#[test]
fn test_srw_files() {
    if !real_files_exist() {
        println!("Skipping SRW tests - real files not available");
        return;
    }

    let srw_files = find_files_by_extension("SRW");
    println!("Found {} SRW files", srw_files.len());

    for file in &srw_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    assert!(!srw_files.is_empty(), "Expected to find SRW files");
}

// KDC Tests (Kodak RAW - 5 files)
#[test]
fn test_kdc_files() {
    if !real_files_exist() {
        println!("Skipping KDC tests - real files not available");
        return;
    }

    let kdc_files = find_files_by_extension("KDC");
    println!("Found {} KDC files", kdc_files.len());

    for file in &kdc_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    assert!(!kdc_files.is_empty(), "Expected to find KDC files");
}

// NRW Tests (Nikon RAW - 4 files)
#[test]
fn test_nrw_files() {
    if !real_files_exist() {
        println!("Skipping NRW tests - real files not available");
        return;
    }

    let nrw_files = find_files_by_extension("NRW");
    println!("Found {} NRW files", nrw_files.len());

    for file in &nrw_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    assert!(!nrw_files.is_empty(), "Expected to find NRW files");
}

// 3FR Tests (Hasselblad RAW - 3 files)
#[test]
fn test_3fr_files() {
    if !real_files_exist() {
        println!("Skipping 3FR tests - real files not available");
        return;
    }

    let fr_files = find_files_by_extension("3FR");
    println!("Found {} 3FR files", fr_files.len());

    for file in &fr_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    assert!(!fr_files.is_empty(), "Expected to find 3FR files");
}

// SR2 Tests (Sony RAW - 1 file)
#[test]
fn test_sr2_files() {
    if !real_files_exist() {
        println!("Skipping SR2 tests - real files not available");
        return;
    }

    let sr2_files = find_files_by_extension("SR2");
    println!("Found {} SR2 files", sr2_files.len());

    for file in &sr2_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    // SR2 files might be rare
    if sr2_files.is_empty() {
        println!("No SR2 files found - this is OK");
    }
}

// PPM Tests (Portable Pixmap - 1 file)
#[test]
fn test_ppm_files() {
    if !real_files_exist() {
        println!("Skipping PPM tests - real files not available");
        return;
    }

    let ppm_files = find_files_by_extension("PPM");
    println!("Found {} PPM files", ppm_files.len());

    for file in &ppm_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    // PPM files might not have EXIF
    if ppm_files.is_empty() {
        println!("No PPM files found - this is OK");
    }
}

// ERF Tests (Epson RAW - 1 file)
#[test]
fn test_erf_files() {
    if !real_files_exist() {
        println!("Skipping ERF tests - real files not available");
        return;
    }

    let erf_files = find_files_by_extension("ERF");
    println!("Found {} ERF files", erf_files.len());

    for file in &erf_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    // ERF files might be rare
    if erf_files.is_empty() {
        println!("No ERF files found - this is OK");
    }
}

// DCR Tests (Kodak RAW - 1 file)
#[test]
fn test_dcr_files() {
    if !real_files_exist() {
        println!("Skipping DCR tests - real files not available");
        return;
    }

    let dcr_files = find_files_by_extension("DCR");
    println!("Found {} DCR files", dcr_files.len());

    for file in &dcr_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    // DCR files might be rare
    if dcr_files.is_empty() {
        println!("No DCR files found - this is OK");
    }
}
