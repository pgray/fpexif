// Integration tests with real image files
use fpexif::ExifParser;
use std::path::Path;

/// Helper function to test parsing a real file
fn test_real_file(filename: &str, format_name: &str, min_tags: usize) {
    let test_file = Path::new("test-data").join(filename);

    // Skip test if file doesn't exist (e.g., in minimal test environments)
    if !test_file.exists() {
        eprintln!(
            "Skipping test - {} file not found at {:?}",
            format_name, test_file
        );
        return;
    }

    let parser = ExifParser::new();
    let result = parser.parse_file(&test_file);

    assert!(
        result.is_ok(),
        "Failed to parse {} file: {:?}",
        format_name,
        result.err()
    );

    let exif_data = result.unwrap();
    assert!(!exif_data.is_empty(), "EXIF data should not be empty");
    assert!(
        exif_data.len() >= min_tags,
        "{} file should have at least {} tags, found {}",
        format_name,
        min_tags,
        exif_data.len()
    );

    println!(
        "✓ Successfully parsed {} file with {} EXIF tags",
        format_name,
        exif_data.len()
    );
}

#[test]
fn test_parse_real_raf_file() {
    test_real_file("DSCF0062.RAF", "RAF", 50);
}

// Tests for additional formats - add as test files become available

#[test]
#[ignore] // Ignored until test file is added
fn test_parse_real_crw_file() {
    test_real_file("sample.CRW", "CRW", 20);
}

#[test]
#[ignore] // Ignored until test file is added
fn test_parse_real_cr2_file() {
    test_real_file("sample.CR2", "CR2", 50);
}

#[test]
#[ignore] // Ignored until test file is added
fn test_parse_real_nef_file() {
    test_real_file("sample.NEF", "NEF", 50);
}

#[test]
#[ignore] // Ignored until test file is added
fn test_parse_real_arw_file() {
    test_real_file("sample.ARW", "ARW", 50);
}

#[test]
#[ignore] // Ignored until test file is added
fn test_parse_real_dng_file() {
    test_real_file("sample.DNG", "DNG", 40);
}

#[test]
#[ignore] // Ignored until test file is added
fn test_parse_real_png_file() {
    test_real_file("sample.png", "PNG", 10);
}

#[test]
#[ignore] // Ignored until test file is added
fn test_parse_real_webp_file() {
    test_real_file("sample.webp", "WebP", 10);
}

#[test]
#[ignore] // Ignored until test file is added
fn test_parse_real_heic_file() {
    test_real_file("sample.heic", "HEIC", 30);
}

#[test]
fn test_parse_nonexistent_file() {
    let parser = ExifParser::new();
    let result = parser.parse_file(Path::new("test-data/nonexistent.jpg"));

    assert!(result.is_err(), "Should fail on nonexistent file");
}

// Test to discover all available test files
#[test]
fn test_discover_available_files() {
    let test_data_dir = Path::new("test-data");
    if !test_data_dir.exists() {
        println!("test-data directory not found");
        return;
    }

    println!("\n=== Available Test Files ===");
    if let Ok(entries) = std::fs::read_dir(test_data_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_uppercase();
                    if matches!(
                        ext_str.as_str(),
                        "RAF"
                            | "CRW"
                            | "CR2"
                            | "CR3"
                            | "NEF"
                            | "ARW"
                            | "DNG"
                            | "ORF"
                            | "PEF"
                            | "MRW"
                            | "X3F"
                            | "PNG"
                            | "WEBP"
                            | "HEIC"
                            | "AVIF"
                            | "JXL"
                            | "JPG"
                            | "JPEG"
                    ) {
                        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        println!(
                            "  {} - {:.2} MB",
                            path.file_name().unwrap().to_string_lossy(),
                            size as f64 / 1_048_576.0
                        );
                    }
                }
            }
        }
    }
    println!("===========================\n");
}
