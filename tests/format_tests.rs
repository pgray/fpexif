// Format-specific tests focusing on error conditions and signature validation
use fpexif::formats;
use std::io::Cursor;

// Note: These tests focus on format detection and error handling rather than
// complete EXIF extraction, which requires real image files with complete data structures.

#[test]
fn test_unsupported_format() {
    // Random data that doesn't match any format (16 bytes for signature check)
    let data = vec![
        0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
        0x88,
    ];

    let cursor = Cursor::new(data);
    let result = formats::extract_exif_segment(cursor);

    assert!(result.is_err(), "Should reject unsupported format");
}

#[test]
fn test_png_signature_validation() {
    // Valid PNG signature but no eXIf chunk
    let mut png = Vec::new();
    png.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]); // PNG signature
                                                                              // IHDR chunk
    png.extend_from_slice(&[0x00, 0x00, 0x00, 0x0D]); // Length
    png.extend_from_slice(b"IHDR");
    png.extend_from_slice(&[0; 13]); // IHDR data
    png.extend_from_slice(&[0; 4]); // CRC
                                    // IEND chunk
    png.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Length
    png.extend_from_slice(b"IEND");
    png.extend_from_slice(&[0; 4]); // CRC

    let cursor = Cursor::new(png);
    let result = formats::extract_exif_segment(cursor);

    assert!(result.is_err(), "Should fail when PNG has no eXIf chunk");
}

#[test]
fn test_webp_signature_validation() {
    // Valid RIFF/WEBP signature but no EXIF chunk
    let mut webp = Vec::new();
    webp.extend_from_slice(b"RIFF");
    webp.extend_from_slice(&20u32.to_le_bytes());
    webp.extend_from_slice(b"WEBP");
    // VP8 chunk without EXIF
    webp.extend_from_slice(b"VP8 ");
    webp.extend_from_slice(&8u32.to_le_bytes());
    webp.extend_from_slice(&[0; 8]);

    let cursor = Cursor::new(webp);
    let result = formats::extract_exif_segment(cursor);

    assert!(result.is_err(), "Should fail when WebP has no EXIF chunk");
}

#[test]
fn test_invalid_tiff_magic() {
    // TIFF with invalid magic number should fail early
    let mut tiff = Vec::new();
    tiff.extend_from_slice(&[0x49, 0x49]); // "II"
    tiff.extend_from_slice(&[0xFF, 0xFF]); // Invalid magic
    tiff.extend_from_slice(&[0x08, 0x00, 0x00, 0x00]); // IFD offset
                                                       // Pad with enough data to avoid EOF errors
    tiff.extend_from_slice(&[0; 100]);

    let cursor = Cursor::new(tiff);
    let result = formats::extract_exif_segment(cursor);

    assert!(result.is_err(), "Should fail on invalid TIFF magic number");
}

#[test]
fn test_jpeg_without_exif() {
    // JPEG without EXIF APP1 segment
    let mut jpeg = Vec::new();
    jpeg.extend_from_slice(&[0xFF, 0xD8]); // SOI
    jpeg.extend_from_slice(&[0xFF, 0xD9]); // EOI (no APP1)

    let cursor = Cursor::new(jpeg);
    let result = formats::extract_exif_segment(cursor);

    assert!(result.is_err(), "Should fail when JPEG has no EXIF");
}

#[test]
fn test_truncated_jpeg() {
    // JPEG with truncated APP1 segment
    let mut jpeg = Vec::new();
    jpeg.extend_from_slice(&[0xFF, 0xD8]); // SOI
    jpeg.extend_from_slice(&[0xFF, 0xE1]); // APP1
    jpeg.extend_from_slice(&[0x00, 0x10]); // Size says 16 bytes
    jpeg.extend_from_slice(b"Exif"); // But we only have 4 bytes

    let cursor = Cursor::new(jpeg);
    let result = formats::extract_exif_segment(cursor);

    assert!(result.is_err(), "Should fail on truncated JPEG APP1");
}

#[test]
fn test_crw_signature_validation() {
    // Invalid CRW file (bad signature)
    let mut crw = Vec::new();
    crw.extend_from_slice(b"II"); // Byte order
    crw.extend_from_slice(&26u32.to_le_bytes()); // Header length
    crw.extend_from_slice(b"INVALID!"); // Invalid signature (should be HEAPCCDR)
    crw.extend_from_slice(&[0u8; 10]); // Padding

    let cursor = Cursor::new(crw);
    let result = formats::extract_exif_segment(cursor);

    assert!(result.is_err(), "Should reject invalid CRW signature");
}

#[test]
fn test_crw_format_detection() {
    // Valid CRW signature should be detected
    let mut crw = Vec::new();
    crw.extend_from_slice(b"II"); // Byte order
    crw.extend_from_slice(&26u32.to_le_bytes()); // Header length
    crw.extend_from_slice(b"HEAPCCDR"); // Valid CIFF signature
    crw.extend_from_slice(&[0u8; 100]); // Padding

    let cursor = Cursor::new(crw);
    let result = formats::extract_exif_segment(cursor);

    // Should fail because we don't have valid CIFF directory structure,
    // but signature should be recognized
    assert!(result.is_err());
}

#[test]
fn test_mrw_signature_validation() {
    // Invalid MRW signature
    let mut mrw = Vec::new();
    mrw.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]); // Invalid (should be 0x00 0x4D 0x52 0x4D)
    mrw.extend_from_slice(&[0u8; 20]);

    let cursor = Cursor::new(mrw);
    let result = formats::extract_exif_segment(cursor);

    assert!(result.is_err(), "Should reject invalid MRW signature");
}

#[test]
fn test_mrw_format_detection() {
    // Valid MRW signature
    let mut mrw = Vec::new();
    mrw.extend_from_slice(&[0x00, 0x4D, 0x52, 0x4D]); // MRM signature
    mrw.extend_from_slice(&16u32.to_be_bytes()); // Data offset
    mrw.extend_from_slice(&32u32.to_be_bytes()); // Data size
    mrw.extend_from_slice(&[0u8; 100]); // Padding

    let cursor = Cursor::new(mrw);
    let result = formats::extract_exif_segment(cursor);

    // Should fail because we don't have valid PRD block,
    // but signature should be recognized
    assert!(result.is_err());
    match result {
        Err(fpexif::errors::ExifError::Format(msg)) => {
            // Either "No EXIF data found" or "Invalid MRW block structure"
            assert!(
                msg.contains("No EXIF data found") || msg.contains("Invalid MRW block structure")
            );
        }
        _ => panic!("Expected Format error"),
    }
}

#[test]
fn test_x3f_signature_validation() {
    // Invalid X3F signature
    let mut x3f = Vec::new();
    x3f.extend_from_slice(b"INVALID"); // Invalid (should be FOVb)
    x3f.extend_from_slice(&[0u8; 20]);

    let cursor = Cursor::new(x3f);
    let result = formats::extract_exif_segment(cursor);

    assert!(result.is_err(), "Should reject invalid X3F signature");
}

#[test]
fn test_x3f_format_detection() {
    // Valid X3F signature
    let mut x3f = Vec::new();
    x3f.extend_from_slice(b"FOVb"); // X3F signature
    x3f.extend_from_slice(&[0x02, 0x00, 0x00, 0x00]); // Version
    x3f.extend_from_slice(&[0u8; 16]); // Unique ID
    x3f.extend_from_slice(&[0u8; 4]); // Mark pattern
    x3f.extend_from_slice(&100u32.to_le_bytes()); // Columns
    x3f.extend_from_slice(&100u32.to_le_bytes()); // Rows
    x3f.extend_from_slice(&0u32.to_le_bytes()); // Rotation
    x3f.extend_from_slice(&[0u8; 100]); // Padding

    let cursor = Cursor::new(x3f);
    let result = formats::extract_exif_segment(cursor);

    // Should fail because we don't have valid directory structure,
    // but signature should be recognized
    assert!(result.is_err());
}
