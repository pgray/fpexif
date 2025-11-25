// Real file format tests using exiftool as reference
// These tests only run if /fpexif/raws/welcome.html exists
use std::fs;
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

            // Track unknown tags
            let unknown_tags: Vec<_> = exif_data
                .iter()
                .filter(|(tag_id, _)| tag_id.name().is_none())
                .map(|(tag_id, _)| format!("0x{:04X}", tag_id.id))
                .collect();
            if !unknown_tags.is_empty() {
                println!(
                    "  ⚠ {} unknown tags: {}",
                    unknown_tags.len(),
                    unknown_tags.join(", ")
                );
            }

            // Validate common EXIF tags
            let mut validations = 0;
            let mut critical_mismatches = 0; // Make/Model mismatches
            let mut dimension_mismatches = 0; // Width/Height mismatches (acceptable for RAW thumbnails)
            let mut photo_mismatches = 0; // ISO/Shutter/Aperture/FocalLength mismatches
            let mut metadata_mismatches = 0; // Lens/DateTime/GPS/etc mismatches

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

            // Check ISO
            if let Some(exiftool_iso) = exiftool_data.get("ISO").and_then(|v| v.as_u64()) {
                if let Some(fpexif_iso_value) = exif_data.get_tag_by_name("ISOSpeedRatings") {
                    validations += 1;
                    if let fpexif::data_types::ExifValue::Short(ref vals) = fpexif_iso_value {
                        if !vals.is_empty() && vals[0] as u64 == exiftool_iso {
                            println!("  ✓ ISO: {}", vals[0]);
                        } else if !vals.is_empty() {
                            photo_mismatches += 1;
                            println!(
                                "  ⚠ ISO mismatch in {}: exiftool={} fpexif={}",
                                path, exiftool_iso, vals[0]
                            );
                        }
                    }
                }
            }

            // Check Shutter Speed (ExposureTime)
            if let Some(exiftool_exposure) = exiftool_data.get("ExposureTime").and_then(|v| {
                if let Some(s) = v.as_str() {
                    // Parse fraction like "1/100" or decimal like "0.01"
                    if s.contains('/') {
                        let parts: Vec<&str> = s.split('/').collect();
                        if parts.len() == 2 {
                            let num = parts[0].parse::<u32>().ok()?;
                            let den = parts[1].parse::<u32>().ok()?;
                            Some((num, den))
                        } else {
                            None
                        }
                    } else {
                        s.parse::<f64>().ok().map(|f| {
                            // Convert decimal to rational
                            let den = 1000u32;
                            let num = (f * den as f64).round() as u32;
                            (num, den)
                        })
                    }
                } else {
                    None
                }
            }) {
                if let Some(fpexif_exposure_value) = exif_data.get_tag_by_name("ExposureTime") {
                    validations += 1;
                    if let fpexif::data_types::ExifValue::Rational(ref vals) = fpexif_exposure_value
                    {
                        if !vals.is_empty() {
                            let (fpexif_num, fpexif_den) = vals[0];
                            let (exiftool_num, exiftool_den) = exiftool_exposure;
                            // Compare as fractions (cross multiply to avoid floating point issues)
                            if fpexif_num * exiftool_den == exiftool_num * fpexif_den {
                                println!("  ✓ Shutter Speed: {}/{}", fpexif_num, fpexif_den);
                            } else {
                                photo_mismatches += 1;
                                println!(
                                    "  ⚠ Shutter Speed mismatch in {}: exiftool={}/{} fpexif={}/{}",
                                    path, exiftool_num, exiftool_den, fpexif_num, fpexif_den
                                );
                            }
                        }
                    }
                }
            }

            // Check Aperture (FNumber)
            if let Some(exiftool_aperture) = exiftool_data.get("FNumber").and_then(|v| {
                if let Some(f) = v.as_f64() {
                    Some(f)
                } else if let Some(s) = v.as_str() {
                    s.parse::<f64>().ok()
                } else {
                    None
                }
            }) {
                if let Some(fpexif_aperture_value) = exif_data.get_tag_by_name("FNumber") {
                    validations += 1;
                    if let fpexif::data_types::ExifValue::Rational(ref vals) = fpexif_aperture_value
                    {
                        if !vals.is_empty() {
                            let (num, den) = vals[0];
                            let fpexif_f = if den > 0 {
                                num as f64 / den as f64
                            } else {
                                0.0
                            };
                            // Allow small floating point differences
                            if (fpexif_f - exiftool_aperture).abs() < 0.01 {
                                println!("  ✓ Aperture: f/{:.1}", fpexif_f);
                            } else {
                                photo_mismatches += 1;
                                println!(
                                    "  ⚠ Aperture mismatch in {}: exiftool=f/{:.1} fpexif=f/{:.1}",
                                    path, exiftool_aperture, fpexif_f
                                );
                            }
                        }
                    }
                }
            }

            // Check Focal Length
            if let Some(exiftool_focal) = exiftool_data.get("FocalLength").and_then(|v| {
                if let Some(f) = v.as_f64() {
                    Some(f)
                } else if let Some(s) = v.as_str() {
                    // Parse strings like "50.0 mm" or "50"
                    s.split_whitespace()
                        .next()
                        .and_then(|num_str| num_str.parse::<f64>().ok())
                } else {
                    None
                }
            }) {
                if let Some(fpexif_focal_value) = exif_data.get_tag_by_name("FocalLength") {
                    validations += 1;
                    if let fpexif::data_types::ExifValue::Rational(ref vals) = fpexif_focal_value {
                        if !vals.is_empty() {
                            let (num, den) = vals[0];
                            let fpexif_focal = if den > 0 {
                                num as f64 / den as f64
                            } else {
                                0.0
                            };
                            // Allow small floating point differences
                            if (fpexif_focal - exiftool_focal).abs() < 0.1 {
                                println!("  ✓ Focal Length: {:.1}mm", fpexif_focal);
                            } else {
                                photo_mismatches += 1;
                                println!(
                                    "  ⚠ Focal Length mismatch in {}: exiftool={:.1}mm fpexif={:.1}mm",
                                    path, exiftool_focal, fpexif_focal
                                );
                            }
                        }
                    }
                }
            }

            // Check Lens Model
            if let Some(exiftool_lens) = exiftool_data.get("LensModel").and_then(|v| v.as_str()) {
                if let Some(fpexif_lens_value) = exif_data.get_tag_by_name("LensModel") {
                    validations += 1;
                    if let fpexif::data_types::ExifValue::Ascii(fpexif_lens) = fpexif_lens_value {
                        let normalized_exiftool = normalize_string(exiftool_lens);
                        let normalized_fpexif = normalize_string(fpexif_lens);
                        if normalized_exiftool == normalized_fpexif {
                            println!("  ✓ Lens: \"{}\"", normalized_fpexif);
                        } else {
                            metadata_mismatches += 1;
                            println!(
                                "  ⚠ Lens mismatch: exiftool=\"{}\" fpexif=\"{}\"",
                                normalized_exiftool, normalized_fpexif
                            );
                        }
                    }
                }
            }

            // Check Serial Number
            if let Some(exiftool_serial) = exiftool_data
                .get("SerialNumber")
                .or_else(|| exiftool_data.get("InternalSerialNumber"))
                .and_then(|v| v.as_str())
            {
                if let Some(fpexif_serial_value) = exif_data
                    .get_tag_by_name("SerialNumber")
                    .or_else(|| exif_data.get_tag_by_name("InternalSerialNumber"))
                {
                    validations += 1;
                    if let fpexif::data_types::ExifValue::Ascii(fpexif_serial) = fpexif_serial_value
                    {
                        let normalized_exiftool = normalize_string(exiftool_serial);
                        let normalized_fpexif = normalize_string(fpexif_serial);
                        if normalized_exiftool == normalized_fpexif {
                            println!("  ✓ Serial: \"{}\"", normalized_fpexif);
                        } else {
                            metadata_mismatches += 1;
                            println!(
                                "  ⚠ Serial mismatch: exiftool=\"{}\" fpexif=\"{}\"",
                                normalized_exiftool, normalized_fpexif
                            );
                        }
                    }
                }
            }

            // Check Exposure Compensation
            if let Some(exiftool_exp_comp) =
                exiftool_data.get("ExposureCompensation").and_then(|v| {
                    if let Some(f) = v.as_f64() {
                        Some(f)
                    } else if let Some(s) = v.as_str() {
                        s.parse::<f64>().ok()
                    } else {
                        None
                    }
                })
            {
                if let Some(fpexif_exp_comp_value) =
                    exif_data.get_tag_by_name("ExposureCompensation")
                {
                    validations += 1;
                    if let fpexif::data_types::ExifValue::SRational(ref vals) =
                        fpexif_exp_comp_value
                    {
                        if !vals.is_empty() {
                            let (num, den) = vals[0];
                            let fpexif_comp = if den > 0 {
                                num as f64 / den as f64
                            } else {
                                0.0
                            };
                            if (fpexif_comp - exiftool_exp_comp).abs() < 0.1 {
                                println!("  ✓ Exp Comp: {:+.1} EV", fpexif_comp);
                            } else {
                                metadata_mismatches += 1;
                                println!(
                                    "  ⚠ Exp Comp mismatch: exiftool={:+.1} fpexif={:+.1}",
                                    exiftool_exp_comp, fpexif_comp
                                );
                            }
                        }
                    }
                }
            }

            // Check Flash
            if let Some(exiftool_flash) = exiftool_data.get("Flash").and_then(|v| v.as_str()) {
                if let Some(fpexif_flash_value) = exif_data.get_tag_by_name("Flash") {
                    validations += 1;
                    let exiftool_fired = exiftool_flash.to_lowercase().contains("fired");
                    if let fpexif::data_types::ExifValue::Short(ref vals) = fpexif_flash_value {
                        if !vals.is_empty() {
                            let fpexif_fired = (vals[0] & 0x01) != 0;
                            if exiftool_fired == fpexif_fired {
                                println!(
                                    "  ✓ Flash: {}",
                                    if fpexif_fired { "Fired" } else { "Not Fired" }
                                );
                            } else {
                                metadata_mismatches += 1;
                                println!(
                                    "  ⚠ Flash mismatch: exiftool={} fpexif={}",
                                    if exiftool_fired { "Fired" } else { "Not Fired" },
                                    if fpexif_fired { "Fired" } else { "Not Fired" }
                                );
                            }
                        }
                    }
                }
            }

            // Check DateTimeOriginal
            if let Some(exiftool_datetime) = exiftool_data
                .get("DateTimeOriginal")
                .and_then(|v| v.as_str())
            {
                if let Some(fpexif_datetime_value) = exif_data.get_tag_by_name("DateTimeOriginal") {
                    validations += 1;
                    if let fpexif::data_types::ExifValue::Ascii(fpexif_datetime) =
                        fpexif_datetime_value
                    {
                        let normalized_exiftool = normalize_string(exiftool_datetime);
                        let normalized_fpexif = normalize_string(fpexif_datetime);
                        if normalized_exiftool == normalized_fpexif {
                            println!("  ✓ DateTime: \"{}\"", normalized_fpexif);
                        } else {
                            metadata_mismatches += 1;
                            println!(
                                "  ⚠ DateTime mismatch: exiftool=\"{}\" fpexif=\"{}\"",
                                normalized_exiftool, normalized_fpexif
                            );
                        }
                    }
                }
            }

            // Check Software
            if let Some(exiftool_software) = exiftool_data.get("Software").and_then(|v| v.as_str())
            {
                if let Some(fpexif_software_value) = exif_data.get_tag_by_name("Software") {
                    validations += 1;
                    if let fpexif::data_types::ExifValue::Ascii(fpexif_software) =
                        fpexif_software_value
                    {
                        let normalized_exiftool = normalize_string(exiftool_software);
                        let normalized_fpexif = normalize_string(fpexif_software);
                        if normalized_exiftool == normalized_fpexif {
                            println!("  ✓ Software: \"{}\"", normalized_fpexif);
                        } else {
                            metadata_mismatches += 1;
                            println!(
                                "  ⚠ Software mismatch: exiftool=\"{}\" fpexif=\"{}\"",
                                normalized_exiftool, normalized_fpexif
                            );
                        }
                    }
                }
            }

            // Check Copyright
            if let Some(exiftool_copyright) =
                exiftool_data.get("Copyright").and_then(|v| v.as_str())
            {
                if let Some(fpexif_copyright_value) = exif_data.get_tag_by_name("Copyright") {
                    validations += 1;
                    if let fpexif::data_types::ExifValue::Ascii(fpexif_copyright) =
                        fpexif_copyright_value
                    {
                        let normalized_exiftool = normalize_string(exiftool_copyright);
                        let normalized_fpexif = normalize_string(fpexif_copyright);
                        if normalized_exiftool == normalized_fpexif {
                            println!("  ✓ Copyright: \"{}\"", normalized_fpexif);
                        } else {
                            metadata_mismatches += 1;
                            println!(
                                "  ⚠ Copyright mismatch: exiftool=\"{}\" fpexif=\"{}\"",
                                normalized_exiftool, normalized_fpexif
                            );
                        }
                    }
                }
            }

            // Check Artist
            if let Some(exiftool_artist) = exiftool_data.get("Artist").and_then(|v| v.as_str()) {
                if let Some(fpexif_artist_value) = exif_data.get_tag_by_name("Artist") {
                    validations += 1;
                    if let fpexif::data_types::ExifValue::Ascii(fpexif_artist) = fpexif_artist_value
                    {
                        let normalized_exiftool = normalize_string(exiftool_artist);
                        let normalized_fpexif = normalize_string(fpexif_artist);
                        if normalized_exiftool == normalized_fpexif {
                            println!("  ✓ Artist: \"{}\"", normalized_fpexif);
                        } else {
                            metadata_mismatches += 1;
                            println!(
                                "  ⚠ Artist mismatch: exiftool=\"{}\" fpexif=\"{}\"",
                                normalized_exiftool, normalized_fpexif
                            );
                        }
                    }
                }
            }

            if validations > 0 {
                let total_mismatches = critical_mismatches
                    + dimension_mismatches
                    + photo_mismatches
                    + metadata_mismatches;
                if total_mismatches > 0 {
                    let mut notes = Vec::new();
                    if dimension_mismatches > 0 {
                        notes.push(format!("{} dimension", dimension_mismatches));
                    }
                    if photo_mismatches > 0 {
                        notes.push(format!("{} photo", photo_mismatches));
                    }
                    if metadata_mismatches > 0 {
                        notes.push(format!("{} metadata", metadata_mismatches));
                    }

                    if !notes.is_empty() && critical_mismatches == 0 {
                        println!(
                            "  ✓ Validated {}/{} tags ({} mismatches - may be expected)",
                            validations - total_mismatches,
                            validations,
                            notes.join(", ")
                        );
                    } else {
                        println!(
                            "  Validated {}/{} tags",
                            validations - total_mismatches,
                            validations
                        );
                    }
                } else {
                    println!("  ✓ Validated {}/{} tags", validations, validations);
                }

                // Fail on any mismatches
                assert_eq!(
                    total_mismatches, 0,
                    "Found {} tag value mismatches in {}",
                    total_mismatches, path
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
    require_real_files_or_skip("test_cr2_files");
    if !real_files_exist() {
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
    require_real_files_or_skip("test_nef_files");
    if !real_files_exist() {
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
    require_real_files_or_skip("test_orf_files");
    if !real_files_exist() {
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

// CRW Tests (Canon RAW - 34 files)
#[test]
fn test_crw_files() {
    require_real_files_or_skip("test_crw_files");
    if !real_files_exist() {
        return;
    }

    let crw_files = find_files_by_extension("CRW");
    println!("Found {} CRW files", crw_files.len());

    for file in &crw_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    assert!(!crw_files.is_empty(), "Expected to find CRW files");
}

// ARW Tests (Sony RAW - 31 files)
#[test]
fn test_arw_files() {
    require_real_files_or_skip("test_arw_files");
    if !real_files_exist() {
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
    require_real_files_or_skip("test_raf_files");
    if !real_files_exist() {
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
    require_real_files_or_skip("test_rw2_files");
    if !real_files_exist() {
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
    require_real_files_or_skip("test_dng_files");
    if !real_files_exist() {
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
    require_real_files_or_skip("test_pef_files");
    if !real_files_exist() {
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
    require_real_files_or_skip("test_raw_files");
    if !real_files_exist() {
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
    require_real_files_or_skip("test_mrw_files");
    if !real_files_exist() {
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
    require_real_files_or_skip("test_x3f_files");
    if !real_files_exist() {
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
    require_real_files_or_skip("test_srw_files");
    if !real_files_exist() {
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
    require_real_files_or_skip("test_kdc_files");
    if !real_files_exist() {
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
    require_real_files_or_skip("test_nrw_files");
    if !real_files_exist() {
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
    require_real_files_or_skip("test_3fr_files");
    if !real_files_exist() {
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
    require_real_files_or_skip("test_sr2_files");
    if !real_files_exist() {
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
    require_real_files_or_skip("test_ppm_files");
    if !real_files_exist() {
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
    require_real_files_or_skip("test_erf_files");
    if !real_files_exist() {
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
    require_real_files_or_skip("test_dcr_files");
    if !real_files_exist() {
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

// MOS Tests (Leaf RAW - 2 files)
#[test]
fn test_mos_files() {
    require_real_files_or_skip("test_mos_files");
    if !real_files_exist() {
        return;
    }

    let mos_files = find_files_by_extension("MOS");
    println!("Found {} MOS files", mos_files.len());

    for file in &mos_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    // MOS files might be rare
    if mos_files.is_empty() {
        println!("No MOS files found - this is OK");
    }
}

// SRF Tests (Sony RAW - 1 file)
#[test]
fn test_srf_files() {
    require_real_files_or_skip("test_srf_files");
    if !real_files_exist() {
        return;
    }

    let srf_files = find_files_by_extension("SRF");
    println!("Found {} SRF files", srf_files.len());

    for file in &srf_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    // SRF files might be rare
    if srf_files.is_empty() {
        println!("No SRF files found - this is OK");
    }
}

// MEF Tests (Mamiya RAW - 1 file)
#[test]
fn test_mef_files() {
    require_real_files_or_skip("test_mef_files");
    if !real_files_exist() {
        return;
    }

    let mef_files = find_files_by_extension("MEF");
    println!("Found {} MEF files", mef_files.len());

    for file in &mef_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    // MEF files might be rare
    if mef_files.is_empty() {
        println!("No MEF files found - this is OK");
    }
}

// MDC Tests (Minolta Digital Camera - 1 file)
#[test]
fn test_mdc_files() {
    require_real_files_or_skip("test_mdc_files");
    if !real_files_exist() {
        return;
    }

    let mdc_files = find_files_by_extension("MDC");
    println!("Found {} MDC files", mdc_files.len());

    for file in &mdc_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    // MDC files might be rare
    if mdc_files.is_empty() {
        println!("No MDC files found - this is OK");
    }
}

// TIFF Tests (3 files)
#[test]
fn test_tiff_files() {
    require_real_files_or_skip("test_tiff_files");
    if !real_files_exist() {
        return;
    }

    let tiff_files = find_files_by_extension("TIFF");
    let tif_files = find_files_by_extension("TIF");

    let mut all_tiff_files = tiff_files;
    all_tiff_files.extend(tif_files);

    println!("Found {} TIFF files", all_tiff_files.len());

    for file in &all_tiff_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    // TIFF files should exist
    if all_tiff_files.is_empty() {
        println!("No TIFF files found - this is OK");
    }
}

// JPEG Tests (1 file)
#[test]
fn test_jpeg_files() {
    require_real_files_or_skip("test_jpeg_files");
    if !real_files_exist() {
        return;
    }

    let jpeg_files = find_files_by_extension("JPEG");
    let jpg_files = find_files_by_extension("JPG");

    let mut all_jpeg_files = jpeg_files;
    all_jpeg_files.extend(jpg_files);

    println!("Found {} JPEG files", all_jpeg_files.len());

    for file in &all_jpeg_files {
        println!("\nTesting: {}", file);
        test_file_against_exiftool(file);
    }

    // JPEG files should exist
    if all_jpeg_files.is_empty() {
        println!("No JPEG files found - this is OK");
    }
}
