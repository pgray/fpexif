// Integration tests with real image files
use fpexif::ExifParser;
use std::fs;
use std::path::Path;

/// Supported image extensions for testing
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "RAF", "CRW", "CR2", "CR3", "NEF", "ARW", "DNG", "ORF", "PEF", "RW2", "MRW", "X3F", "SRW",
    "KDC", "NRW", "3FR", "SR2", "ERF", "DCR", "MOS", "SRF", "MEF", "MDC", "PNG", "WEBP", "HEIC",
    "AVIF", "JXL", "JPG", "JPEG", "TIFF", "TIF",
];

/// Find all image files in a directory
fn find_image_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && let Some(ext) = path.extension()
            {
                let ext_upper = ext.to_string_lossy().to_uppercase();
                if SUPPORTED_EXTENSIONS.contains(&ext_upper.as_str()) {
                    files.push(path);
                }
            }
        }
    }
    files.sort();
    files
}

/// Test parsing a single image file
fn test_parse_file(path: &Path) {
    let parser = ExifParser::new();
    let result = parser.parse_file(path);

    let filename = path
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".to_string());

    match result {
        Ok(exif_data) => {
            assert!(
                !exif_data.is_empty(),
                "EXIF data should not be empty for {}",
                filename
            );

            // Track unknown tags
            let unknown_tags: Vec<_> = exif_data
                .iter()
                .filter(|(tag_id, _)| tag_id.name().is_none())
                .map(|(tag_id, _)| format!("0x{:04X}", tag_id.id))
                .collect();

            if unknown_tags.is_empty() {
                println!("  {} - {} tags", filename, exif_data.len());
            } else {
                println!(
                    "  {} - {} tags, {} unknown tags: {}",
                    filename,
                    exif_data.len(),
                    unknown_tags.len(),
                    unknown_tags.join(", ")
                );
            }
        }
        Err(e) => {
            panic!("Failed to parse {}: {:?}", filename, e);
        }
    }
}

#[test]
fn test_parse_test_data_files() {
    let test_data_dir = Path::new("test-data");

    if !test_data_dir.exists() {
        if std::env::var("CI").is_ok() {
            panic!("test-data directory not found - required in CI");
        }
        println!("Skipping - test-data directory not found");
        return;
    }

    let files = find_image_files(test_data_dir);

    if files.is_empty() {
        if std::env::var("CI").is_ok() {
            panic!("No image files found in test-data - required in CI");
        }
        println!("No image files found in test-data");
        return;
    }

    println!("\n=== Testing {} files in test-data ===", files.len());
    for file in &files {
        test_parse_file(file);
    }
    println!("=== All {} files parsed successfully ===\n", files.len());
}

#[test]
fn test_parse_nonexistent_file() {
    let parser = ExifParser::new();
    let result = parser.parse_file(Path::new("test-data/nonexistent.jpg"));

    assert!(result.is_err(), "Should fail on nonexistent file");
}
