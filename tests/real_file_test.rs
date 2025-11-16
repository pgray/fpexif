// Integration tests with real image files
use fpexif::ExifParser;
use std::path::Path;

#[test]
fn test_parse_real_raf_file() {
    let test_file = Path::new("test-data/DSCF0062.RAF");

    // Skip test if file doesn't exist (e.g., in minimal test environments)
    if !test_file.exists() {
        eprintln!("Skipping test - RAF file not found at {:?}", test_file);
        return;
    }

    let parser = ExifParser::new();
    let result = parser.parse_file(test_file);

    assert!(result.is_ok(), "Failed to parse RAF file: {:?}", result.err());

    let exif_data = result.unwrap();
    assert!(!exif_data.is_empty(), "EXIF data should not be empty");

    // Verify we can access the data
    println!("Successfully parsed RAF file with {} EXIF tags", exif_data.len());
}

#[test]
fn test_parse_nonexistent_file() {
    let parser = ExifParser::new();
    let result = parser.parse_file(Path::new("test-data/nonexistent.jpg"));

    assert!(result.is_err(), "Should fail on nonexistent file");
}
