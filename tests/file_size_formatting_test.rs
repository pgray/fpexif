// Unit tests for file size formatting

#[cfg(all(feature = "serde", test))]
mod file_size_tests {
    use fpexif::output::to_exiftool_json;
    use fpexif::ExifData;
    use serde_json::Value;

    fn get_file_size_string(bytes: u64) -> String {
        let exif_data = ExifData::new();
        let json = to_exiftool_json(&exif_data, None, Some(bytes));

        if let Value::Array(arr) = &json {
            if let Some(Value::Object(obj)) = arr.first() {
                if let Some(Value::String(size)) = obj.get("FileSize") {
                    return size.clone();
                }
            }
        }
        panic!("FileSize not found in JSON");
    }

    #[test]
    fn test_bytes_formatting() {
        assert_eq!(get_file_size_string(0), "0 bytes");
        assert_eq!(get_file_size_string(1), "1 bytes");
        assert_eq!(get_file_size_string(512), "512 bytes");
        assert_eq!(get_file_size_string(1023), "1023 bytes");
    }

    #[test]
    fn test_kilobytes_formatting() {
        // 1 KB
        assert_eq!(get_file_size_string(1024), "1.0 kB");
        // 1.5 KB
        assert_eq!(get_file_size_string(1536), "1.5 kB");
        // 10 KB
        assert_eq!(get_file_size_string(10240), "10.0 kB");
        // 999.9 KB (just under 1 MB)
        assert_eq!(get_file_size_string(1023 * 1024), "1023.0 kB");
    }

    #[test]
    fn test_megabytes_formatting() {
        // 1 MB - exiftool uses integer MB
        assert_eq!(get_file_size_string(1024 * 1024), "1 MB");
        // 34 MB (from example)
        assert_eq!(get_file_size_string(35651584), "34 MB");
        // 999 MB (just under 1 GB)
        assert_eq!(get_file_size_string(1023 * 1024 * 1024), "1023 MB");
    }

    #[test]
    fn test_gigabytes_formatting() {
        // 1 GB
        assert_eq!(get_file_size_string(1024 * 1024 * 1024), "1.0 GB");
        // 1.5 GB
        assert_eq!(get_file_size_string(1610612736), "1.5 GB");
        // 10 GB
        assert_eq!(get_file_size_string(10 * 1024 * 1024 * 1024), "10.0 GB");
    }

    #[test]
    fn test_file_size_bytes_number() {
        // Test that FileSizeBytes is present as a number
        let exif_data = ExifData::new();
        let test_size = 35651584u64;
        let json = to_exiftool_json(&exif_data, None, Some(test_size));

        if let Value::Array(arr) = &json {
            if let Some(Value::Object(obj)) = arr.first() {
                // Check FileSizeBytes exists
                assert!(
                    obj.contains_key("FileSizeBytes"),
                    "FileSizeBytes should exist"
                );

                // Check it's a number
                if let Some(Value::Number(size)) = obj.get("FileSizeBytes") {
                    assert_eq!(
                        size.as_u64().unwrap(),
                        test_size,
                        "FileSizeBytes should match input"
                    );
                } else {
                    panic!("FileSizeBytes should be a number");
                }
            }
        }
    }
}
