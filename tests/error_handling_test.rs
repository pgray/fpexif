// Error handling tests
use fpexif::{errors::ExifError, ExifParser};

#[test]
fn test_io_error_handling() {
    // Test with data that's too short to read
    let short_data = vec![0xFF];
    let parser = ExifParser::new();

    let result = parser.parse_bytes(&short_data);
    assert!(result.is_err(), "Should handle IO errors");
}

#[test]
fn test_format_error_for_unknown_format() {
    let unknown_format = vec![0x00; 20];
    let parser = ExifParser::new();

    let result = parser.parse_bytes(&unknown_format);
    assert!(result.is_err());

    match result {
        Err(ExifError::Format(_)) => {
            // Expected error type
        }
        Err(e) => panic!("Wrong error type: {:?}", e),
        Ok(_) => panic!("Should have failed"),
    }
}

#[test]
fn test_corrupted_jpeg_marker() {
    let mut jpeg = Vec::new();
    jpeg.extend_from_slice(&[0xFF, 0xD8]); // SOI
    jpeg.extend_from_slice(&[0xFF, 0xE1]); // APP1
    jpeg.extend_from_slice(&[0xFF, 0xFF]); // Invalid size (would overflow)
    jpeg.extend_from_slice(b"Exif\0\0");

    let parser = ExifParser::new();
    let result = parser.parse_bytes(&jpeg);

    assert!(result.is_err(), "Should handle corrupted JPEG markers");
}

#[test]
fn test_exif_without_tiff_header() {
    let mut jpeg = Vec::new();
    jpeg.extend_from_slice(&[0xFF, 0xD8]); // SOI
    jpeg.extend_from_slice(&[0xFF, 0xE1]); // APP1
    jpeg.extend_from_slice(&[0x00, 0x0A]); // Size
    jpeg.extend_from_slice(b"Exif\0\0");
    // Missing TIFF header
    jpeg.extend_from_slice(&[0xFF, 0xD9]); // EOI

    let parser = ExifParser::new();
    let result = parser.parse_bytes(&jpeg);

    assert!(result.is_err(), "Should fail without valid TIFF header");
}

#[test]
fn test_invalid_ifd_offset() {
    let mut jpeg = Vec::new();
    jpeg.extend_from_slice(&[0xFF, 0xD8]); // SOI
    jpeg.extend_from_slice(&[0xFF, 0xE1]); // APP1
    jpeg.extend_from_slice(&[0x00, 0x12]); // Size
    jpeg.extend_from_slice(b"Exif\0\0");
    jpeg.extend_from_slice(&[0x49, 0x49]); // "II"
    jpeg.extend_from_slice(&[0x2A, 0x00]); // Magic
    jpeg.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]); // Invalid IFD offset
    jpeg.extend_from_slice(&[0xFF, 0xD9]); // EOI

    let parser = ExifParser::new();
    let result = parser.parse_bytes(&jpeg);

    assert!(result.is_err(), "Should handle invalid IFD offset");
}

#[test]
fn test_empty_file() {
    let empty = vec![];
    let parser = ExifParser::new();

    let result = parser.parse_bytes(&empty);
    assert!(result.is_err(), "Should fail on empty file");
}

#[test]
fn test_very_large_segment_size() {
    let mut jpeg = Vec::new();
    jpeg.extend_from_slice(&[0xFF, 0xD8]); // SOI
    jpeg.extend_from_slice(&[0xFF, 0xE1]); // APP1
    jpeg.extend_from_slice(&[0xFF, 0xFE]); // Very large size (65534 bytes)
    jpeg.extend_from_slice(b"Exif\0\0");

    let parser = ExifParser::new();
    let result = parser.parse_bytes(&jpeg);

    // Should fail because we can't read that much data
    assert!(result.is_err(), "Should handle oversized segments");
}

#[test]
fn test_invalid_endianness_marker() {
    let mut tiff = Vec::new();
    tiff.extend_from_slice(&[0x58, 0x58]); // Invalid endianness marker
    tiff.extend_from_slice(&[0x2A, 0x00]); // Magic
    tiff.extend_from_slice(&[0x08, 0x00, 0x00, 0x00]); // IFD offset

    let parser = ExifParser::new();
    let result = parser.parse_bytes(&tiff);

    assert!(result.is_err(), "Should reject invalid endianness marker");
}

#[test]
fn test_strict_vs_non_strict_parsing() {
    // Create JPEG with slightly malformed EXIF
    let mut jpeg = Vec::new();
    jpeg.extend_from_slice(&[0xFF, 0xD8]); // SOI
    jpeg.extend_from_slice(&[0xFF, 0xE1]); // APP1
    jpeg.extend_from_slice(&[0x00, 0x10]); // Size
    jpeg.extend_from_slice(b"Exif\0\0");
    jpeg.extend_from_slice(&[0x49, 0x49, 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00]);
    jpeg.extend_from_slice(&[0xFF, 0xD9]); // EOI

    // Strict parser
    let strict_parser = ExifParser::new().strict(true);
    let strict_result = strict_parser.parse_bytes(&jpeg);

    // Non-strict parser
    let lenient_parser = ExifParser::new().strict(false);
    let lenient_result = lenient_parser.parse_bytes(&jpeg);

    // Both might fail or succeed, but they should handle the same data
    assert_eq!(
        strict_result.is_ok(),
        lenient_result.is_ok(),
        "Parsers should behave consistently on valid data"
    );
}

#[test]
fn test_truncated_tiff_data() {
    let mut tiff = Vec::new();
    tiff.extend_from_slice(&[0x49, 0x49]); // "II"
    tiff.extend_from_slice(&[0x2A, 0x00]); // Magic
    tiff.extend_from_slice(&[0x08, 0x00]); // Truncated IFD offset

    let parser = ExifParser::new();
    let result = parser.parse_bytes(&tiff);

    assert!(result.is_err(), "Should fail on truncated TIFF");
}

#[test]
fn test_circular_ifd_reference() {
    // This is a more complex test - create TIFF with circular IFD reference
    let mut tiff = Vec::new();
    tiff.extend_from_slice(&[0x49, 0x49]); // "II"
    tiff.extend_from_slice(&[0x2A, 0x00]); // Magic
    tiff.extend_from_slice(&[0x08, 0x00, 0x00, 0x00]); // IFD offset = 8
    tiff.extend_from_slice(&[0x00, 0x00]); // 0 entries
    tiff.extend_from_slice(&[0x08, 0x00, 0x00, 0x00]); // Next IFD = 8 (circular!)

    let parser = ExifParser::new();
    let result = parser.parse_bytes(&tiff);

    // The parser should either detect the circular reference or limit iterations
    // Either way, it shouldn't hang
    let _ = result; // We just want to ensure it completes
}
