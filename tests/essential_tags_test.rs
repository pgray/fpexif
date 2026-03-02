// Tests for essential tags and WASM compatibility features

use fpexif::ExifParser;
#[cfg(feature = "serde")]
use fpexif::output::{get_tag_value, to_exiftool_json};
use serde_json::Value;
use std::path::PathBuf;

#[cfg(feature = "serde")]
#[test]
fn test_file_size_formatting() {
    // Test file size formatting with various byte counts
    let parser = ExifParser::new();

    // Use a real test file
    let test_file = PathBuf::from("/fpexif/raws/RAW_CANON_1000D.CR2");
    if !test_file.exists() {
        println!("Skipping test: test file not found");
        return;
    }

    // Read the file to get its size
    let file_data = std::fs::read(&test_file).expect("Failed to read test file");
    let file_size = file_data.len() as u64;

    match parser.parse_bytes(&file_data) {
        Ok(exif_data) => {
            let json = to_exiftool_json(&exif_data, None, Some(file_size));

            // Extract the object from the array
            if let Value::Array(arr) = &json {
                if let Some(Value::Object(obj)) = arr.first() {
                    // Check FileSize (human-readable) exists
                    assert!(
                        obj.contains_key("FileSize"),
                        "FileSize field should be present"
                    );

                    // Check FileSizeBytes (numeric) exists
                    assert!(
                        obj.contains_key("FileSizeBytes"),
                        "FileSizeBytes field should be present"
                    );

                    // Verify FileSizeBytes matches input
                    if let Some(Value::Number(size_bytes)) = obj.get("FileSizeBytes") {
                        assert_eq!(
                            size_bytes.as_u64().unwrap(),
                            file_size,
                            "FileSizeBytes should match input file size"
                        );
                    } else {
                        panic!("FileSizeBytes should be a number");
                    }

                    // Verify FileSize is a human-readable string
                    if let Some(Value::String(size_str)) = obj.get("FileSize") {
                        // Should contain either "bytes", "kB", "MB", or "GB"
                        assert!(
                            size_str.contains("bytes")
                                || size_str.contains("kB")
                                || size_str.contains("MB")
                                || size_str.contains("GB"),
                            "FileSize should be human-readable with units"
                        );
                    } else {
                        panic!("FileSize should be a string");
                    }
                } else {
                    panic!("Expected JSON object in array");
                }
            } else {
                panic!("Expected JSON array");
            }
        }
        Err(e) => panic!("Failed to parse EXIF data: {}", e),
    }
}

#[cfg(feature = "serde")]
#[test]
fn test_case_insensitive_tag_lookup() {
    // Test case-insensitive tag lookup
    let parser = ExifParser::new();

    let test_file = PathBuf::from("/fpexif/raws/RAW_CANON_1000D.CR2");
    if !test_file.exists() {
        println!("Skipping test: test file not found");
        return;
    }

    let file_data = std::fs::read(&test_file).expect("Failed to read test file");
    let file_size = file_data.len() as u64;

    match parser.parse_bytes(&file_data) {
        Ok(exif_data) => {
            let json = to_exiftool_json(&exif_data, None, Some(file_size));

            // Test "Make" with various casings
            let make_variants = vec!["Make", "make", "MAKE", "MaKe"];
            for variant in make_variants {
                let value = get_tag_value(&json, variant);
                assert!(
                    value.is_some(),
                    "Should find 'Make' tag with variant: {}",
                    variant
                );

                // All variants should return the same value
                if let Some(Value::String(s)) = value {
                    assert!(
                        !s.is_empty(),
                        "Make value should not be empty for variant: {}",
                        variant
                    );
                }
            }

            // Test "Model" with various casings
            let model_variants = vec!["Model", "model", "MODEL"];
            for variant in model_variants {
                let value = get_tag_value(&json, variant);
                assert!(
                    value.is_some(),
                    "Should find 'Model' tag with variant: {}",
                    variant
                );
            }

            // Test tags that might not exist
            let missing = get_tag_value(&json, "NonExistentTag");
            assert!(missing.is_none(), "Should return None for missing tags");
        }
        Err(e) => panic!("Failed to parse EXIF data: {}", e),
    }
}

#[cfg(feature = "serde")]
#[test]
fn test_essential_tags_presence() {
    // Test that all 8 essential tags are either present or gracefully absent
    let parser = ExifParser::new();

    let test_file = PathBuf::from("/fpexif/raws/RAW_CANON_1000D.CR2");
    if !test_file.exists() {
        println!("Skipping test: test file not found");
        return;
    }

    let file_data = std::fs::read(&test_file).expect("Failed to read test file");
    let file_size = file_data.len() as u64;

    match parser.parse_bytes(&file_data) {
        Ok(exif_data) => {
            let json = to_exiftool_json(&exif_data, None, Some(file_size));

            // Essential tags to check (some may not be present in all images)
            let essential_tags = vec![
                "Make",
                "Model",
                "ISO",
                "Aperture",
                "ShutterSpeed",
                "FocalLength",
                "CreateDate",
                "FileSize",
            ];

            println!("Checking essential tags:");
            for tag in essential_tags {
                let value = get_tag_value(&json, tag);
                match value {
                    Some(v) => println!("  ✓ {}: {}", tag, v),
                    None => println!("  - {} (not present)", tag),
                }
            }

            // FileSize should ALWAYS be present when we provide it
            let file_size_value = get_tag_value(&json, "FileSize");
            assert!(
                file_size_value.is_some(),
                "FileSize should always be present when provided"
            );
        }
        Err(e) => panic!("Failed to parse EXIF data: {}", e),
    }
}

#[cfg(feature = "serde")]
#[test]
fn test_snake_case_tag_lookup() {
    // Test that snake_case tag names work (e.g., "shutter_speed" for "ShutterSpeed")
    let parser = ExifParser::new();

    let test_file = PathBuf::from("/fpexif/raws/RAW_CANON_1000D.CR2");
    if !test_file.exists() {
        println!("Skipping test: test file not found");
        return;
    }

    let file_data = std::fs::read(&test_file).expect("Failed to read test file");
    let file_size = file_data.len() as u64;

    match parser.parse_bytes(&file_data) {
        Ok(exif_data) => {
            let json = to_exiftool_json(&exif_data, None, Some(file_size));

            // Test snake_case variants
            let test_cases = vec![
                ("shutter_speed", "ShutterSpeed"),
                ("focal_length", "FocalLength"),
                ("file_size", "FileSize"),
            ];

            for (snake_case, pascal_case) in test_cases {
                let snake_value = get_tag_value(&json, snake_case);
                let pascal_value = get_tag_value(&json, pascal_case);

                // Both should find the same value (or both should be None)
                match (snake_value, pascal_value) {
                    (Some(v1), Some(v2)) => {
                        assert_eq!(
                            v1, v2,
                            "snake_case and PascalCase should return same value for {}",
                            pascal_case
                        );
                    }
                    (None, None) => {
                        // Both missing is OK
                    }
                    _ => {
                        panic!("snake_case and PascalCase should both find or both miss");
                    }
                }
            }
        }
        Err(e) => panic!("Failed to parse EXIF data: {}", e),
    }
}

#[cfg(feature = "serde")]
#[test]
fn test_file_size_without_parameter() {
    // Test that when file_size is not provided, FileSize tag is absent
    let parser = ExifParser::new();

    let test_file = PathBuf::from("/fpexif/raws/RAW_CANON_1000D.CR2");
    if !test_file.exists() {
        println!("Skipping test: test file not found");
        return;
    }

    let file_data = std::fs::read(&test_file).expect("Failed to read test file");

    match parser.parse_bytes(&file_data) {
        Ok(exif_data) => {
            // Call without file_size parameter (backward compatibility)
            let json = to_exiftool_json(&exif_data, None, None);

            // FileSize should NOT be present
            let file_size_value = get_tag_value(&json, "FileSize");
            assert!(
                file_size_value.is_none(),
                "FileSize should be absent when not provided"
            );
        }
        Err(e) => panic!("Failed to parse EXIF data: {}", e),
    }
}
