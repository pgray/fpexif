// Test FileSize tag across all major camera manufacturers

use fpexif::ExifParser;
#[cfg(feature = "serde")]
use fpexif::output::{get_tag_value, to_exiftool_json};
use std::fs;
use std::path::Path;

#[cfg(feature = "serde")]
#[test]
fn test_filesize_across_all_manufacturers() {
    // Test files from different manufacturers
    let test_files = vec![
        // Canon
        ("/fpexif/raws/RAW_CANON_1000D.CR2", "Canon"),
        // Nikon
        ("/fpexif/raws/RAW_NIKON_D3.NEF", "Nikon"),
        // Sony
        ("/fpexif/raws/RAW_SONY_A100.ARW", "Sony"),
        // Fujifilm
        ("/fpexif/raws/RAW_FUJI_E550.RAF", "Fujifilm"),
        // Olympus
        ("/fpexif/raws/RAW_OLYMPUS_E30.ORF", "Olympus"),
        // Panasonic
        ("/fpexif/raws/RAW_PANASONIC_G1.RW2", "Panasonic"),
        // Pentax
        ("/fpexif/raws/RAW_PENTAX_K10D.PEF", "Pentax"),
        // Minolta
        ("/fpexif/raws/RAW_MINOLTA_DYNAX5D.MRW", "Minolta"),
        // Sigma
        ("/fpexif/raws/RAW_SIGMA_SD14.X3F", "Sigma"),
    ];

    let parser = ExifParser::new();
    println!("\n=== Testing FileSize across manufacturers ===");

    let mut tested = 0;
    let mut passed = 0;

    for (file_path, manufacturer) in test_files {
        let path = Path::new(file_path);
        if !path.exists() {
            println!("  - {} (file not available)", manufacturer);
            continue;
        }

        tested += 1;

        let file_data = match fs::read(path) {
            Ok(data) => data,
            Err(e) => {
                println!("  ✗ {} - Failed to read: {}", manufacturer, e);
                continue;
            }
        };

        let file_size = file_data.len() as u64;

        match parser.parse_bytes(&file_data) {
            Ok(exif_data) => {
                let json = to_exiftool_json(&exif_data, None, Some(file_size));

                // Check FileSize presence
                match (
                    get_tag_value(&json, "FileSize"),
                    get_tag_value(&json, "FileSizeBytes"),
                ) {
                    (Some(filesize_value), Some(serde_json::Value::Number(bytes))) => {
                        passed += 1;

                        // Verify the values
                        assert_eq!(
                            bytes.as_u64().unwrap(),
                            file_size,
                            "{}: FileSizeBytes should match actual file size",
                            manufacturer
                        );

                        println!(
                            "  ✓ {} - FileSize: {}, FileSizeBytes: {}",
                            manufacturer, filesize_value, file_size
                        );
                    }
                    _ => {
                        println!("  ✗ {} - FileSize tag missing!", manufacturer);
                    }
                }
            }
            Err(e) => {
                println!("  ✗ {} - Parse error: {}", manufacturer, e);
            }
        }
    }

    println!(
        "\nResult: {}/{} manufacturers tested successfully",
        passed, tested
    );

    // All tested files should have FileSize
    assert_eq!(passed, tested, "All tested files should have FileSize tag");
}

#[cfg(feature = "serde")]
#[test]
fn test_filesize_formatting_consistency() {
    // Test that FileSize formatting is consistent across files of similar sizes
    let test_files = vec![
        "/fpexif/raws/RAW_CANON_1000D.CR2",
        "/fpexif/raws/RAW_NIKON_D3.NEF",
        "/fpexif/raws/RAW_SONY_A100.ARW",
    ];

    let parser = ExifParser::new();
    println!("\n=== Testing FileSize formatting consistency ===");

    for file_path in test_files {
        let path = Path::new(file_path);
        if !path.exists() {
            continue;
        }

        let file_data = fs::read(path).expect("Failed to read file");
        let file_size = file_data.len() as u64;

        if let Ok(exif_data) = parser.parse_bytes(&file_data) {
            let json = to_exiftool_json(&exif_data, None, Some(file_size));

            let filesize_str = get_tag_value(&json, "FileSize");
            let filesize_bytes = get_tag_value(&json, "FileSizeBytes");

            let filename = path.file_name().unwrap().to_string_lossy();

            if let (
                Some(serde_json::Value::String(size_str)),
                Some(serde_json::Value::Number(bytes)),
            ) = (filesize_str, filesize_bytes)
            {
                println!(
                    "  {} - {} ({} bytes)",
                    filename,
                    size_str,
                    bytes.as_u64().unwrap()
                );

                // Verify format is correct (should contain units)
                assert!(
                    size_str.contains("MB")
                        || size_str.contains("kB")
                        || size_str.contains("bytes")
                        || size_str.contains("GB"),
                    "FileSize should contain units"
                );

                // Verify bytes match actual file size
                assert_eq!(
                    bytes.as_u64().unwrap(),
                    file_size,
                    "FileSizeBytes should match actual file size"
                );
            }
        }
    }
}
