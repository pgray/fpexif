// Integration tests for fpexif library
use fpexif::{ExifData, ExifParser};
use std::io::Cursor;

/// Helper to create a minimal valid JPEG with EXIF
fn create_test_jpeg_with_exif() -> Vec<u8> {
    let mut jpeg = Vec::new();

    // JPEG SOI marker
    jpeg.extend_from_slice(&[0xFF, 0xD8]);

    // APP1 marker (EXIF)
    jpeg.extend_from_slice(&[0xFF, 0xE1]);

    // APP1 segment size (including size field)
    let app1_size = 2 + 6 + 8 + 2 + 12 + 4; // size + "Exif\0\0" + TIFF header + 1 IFD entry + next IFD
    jpeg.extend_from_slice(&(app1_size as u16).to_be_bytes());

    // EXIF identifier
    jpeg.extend_from_slice(b"Exif\0\0");

    // TIFF header (little-endian)
    jpeg.extend_from_slice(&[0x49, 0x49]); // "II" - little-endian
    jpeg.extend_from_slice(&[0x2A, 0x00]); // TIFF magic number
    jpeg.extend_from_slice(&[0x08, 0x00, 0x00, 0x00]); // Offset to first IFD

    // IFD0: 1 entry
    jpeg.extend_from_slice(&[0x01, 0x00]); // Number of entries

    // Tag entry: Orientation (0x0112)
    jpeg.extend_from_slice(&[0x12, 0x01]); // Tag ID
    jpeg.extend_from_slice(&[0x03, 0x00]); // Type: SHORT
    jpeg.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // Count: 1
    jpeg.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // Value: 1

    // Next IFD offset (0 = no more IFDs)
    jpeg.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

    // JPEG EOI marker
    jpeg.extend_from_slice(&[0xFF, 0xD9]);

    jpeg
}

#[test]
fn test_parse_jpeg_with_exif() {
    let jpeg_data = create_test_jpeg_with_exif();
    let parser = ExifParser::new();

    let result = parser.parse_bytes(&jpeg_data);
    assert!(result.is_ok(), "Failed to parse JPEG with EXIF");

    if let Ok(exif_data) = result {
        assert!(!exif_data.is_empty(), "EXIF data should not be empty");
    }
}

#[test]
fn test_parse_invalid_data() {
    let invalid_data = vec![0x00, 0x00, 0x00, 0x00];
    let parser = ExifParser::new();

    let result = parser.parse_bytes(&invalid_data);
    assert!(result.is_err(), "Should fail on invalid data");
}

#[test]
fn test_parse_empty_data() {
    let empty_data = vec![];
    let parser = ExifParser::new();

    let result = parser.parse_bytes(&empty_data);
    assert!(result.is_err(), "Should fail on empty data");
}

#[test]
fn test_parser_default() {
    let parser1 = ExifParser::new();
    let parser2 = ExifParser::default();

    // Both should work the same way
    let jpeg_data = create_test_jpeg_with_exif();
    assert!(parser1.parse_bytes(&jpeg_data).is_ok());
    assert!(parser2.parse_bytes(&jpeg_data).is_ok());
}

#[test]
fn test_exif_data_default() {
    let exif1 = ExifData::new();
    let exif2 = ExifData::default();

    assert_eq!(exif1.len(), exif2.len());
    assert!(exif1.is_empty());
    assert!(exif2.is_empty());
}

#[test]
fn test_parser_with_cursor() {
    let jpeg_data = create_test_jpeg_with_exif();
    let cursor = Cursor::new(jpeg_data);
    let parser = ExifParser::new();

    let result = parser.parse_reader(cursor);
    assert!(result.is_ok(), "Should parse from cursor");
}

#[test]
fn test_jpeg_without_exif() {
    // Minimal JPEG without EXIF
    let jpeg = vec![
        0xFF, 0xD8, // SOI
        0xFF, 0xD9, // EOI
    ];

    let parser = ExifParser::new();
    let result = parser.parse_bytes(&jpeg);

    // Should fail because there's no EXIF data
    assert!(result.is_err(), "Should fail when no EXIF data present");
}

#[test]
fn test_truncated_jpeg() {
    // Truncated JPEG (just SOI marker)
    let jpeg = vec![0xFF, 0xD8];

    let parser = ExifParser::new();
    let result = parser.parse_bytes(&jpeg);

    assert!(result.is_err(), "Should fail on truncated JPEG");
}

#[test]
fn test_malformed_app1_marker() {
    let mut jpeg = Vec::new();
    jpeg.extend_from_slice(&[0xFF, 0xD8]); // SOI
    jpeg.extend_from_slice(&[0xFF, 0xE1]); // APP1 marker
    jpeg.extend_from_slice(&[0x00, 0x02]); // Invalid size (too small)
    jpeg.extend_from_slice(&[0xFF, 0xD9]); // EOI

    let parser = ExifParser::new();
    let result = parser.parse_bytes(&jpeg);

    assert!(result.is_err(), "Should fail on malformed APP1 marker");
}
