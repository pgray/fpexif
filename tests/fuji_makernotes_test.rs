// Fujifilm MakerNotes decoding tests
use fpexif::data_types::ExifValue;
use fpexif::ExifParser;
use std::path::Path;

#[test]
fn test_decode_film_mode() {
    let test_path = Path::new("/fpexif/raws/RAW_FUIJI_X-E2.RAF");
    if !test_path.exists() {
        println!("Skipping test - RAF file not found");
        return;
    }

    let parser = ExifParser::new();
    let result = parser.parse_file(test_path);

    match result {
        Ok(exif_data) => {
            println!("Successfully parsed RAF file with {} tags", exif_data.len());

            // Check if we have maker notes
            if let Some(maker_notes) = exif_data.get_maker_notes() {
                println!("Found {} maker note tags", maker_notes.len());

                // Look for FilmMode (0x1401)
                if let Some(film_mode_tag) = maker_notes.get(&0x1401) {
                    println!("FilmMode tag found: {:?}", film_mode_tag.value);

                    // Should be decoded to an ASCII string, not a raw Short value
                    match &film_mode_tag.value {
                        ExifValue::Ascii(s) => {
                            println!("FilmMode decoded: {}", s);
                            assert!(
                                !s.is_empty() && s != "Unknown",
                                "FilmMode should be decoded to a known value"
                            );
                        }
                        ExifValue::Short(values) => {
                            // If it's still a Short, print the raw value for debugging
                            println!("FilmMode raw value: {:?}", values);
                            panic!("FilmMode should be decoded to Ascii, not Short");
                        }
                        _ => {
                            panic!("FilmMode has unexpected value type");
                        }
                    }
                } else {
                    println!("FilmMode tag (0x1401) not found in this file");
                }
            } else {
                panic!("No maker notes found in RAF file");
            }
        }
        Err(e) => {
            panic!("Failed to parse RAF file: {:?}", e);
        }
    }
}

#[test]
fn test_decode_white_balance() {
    let test_path = Path::new("/fpexif/raws/RAW_FUJI_F700.RAF");
    if !test_path.exists() {
        println!("Skipping test - RAF file not found");
        return;
    }

    let parser = ExifParser::new();
    let result = parser.parse_file(test_path);

    match result {
        Ok(exif_data) => {
            if let Some(maker_notes) = exif_data.get_maker_notes() {
                // Look for WhiteBalance (0x1002)
                if let Some(wb_tag) = maker_notes.get(&0x1002) {
                    println!("WhiteBalance tag found: {:?}", wb_tag.value);

                    match &wb_tag.value {
                        ExifValue::Ascii(s) => {
                            println!("WhiteBalance decoded: {}", s);
                            assert!(!s.is_empty(), "WhiteBalance should be decoded");
                        }
                        _ => {
                            println!("WhiteBalance has value: {:?}", wb_tag.value);
                        }
                    }
                } else {
                    println!("WhiteBalance tag (0x1002) not found in this file");
                }
            }
        }
        Err(e) => {
            panic!("Failed to parse RAF file: {:?}", e);
        }
    }
}

#[test]
fn test_decode_dynamic_range() {
    let test_path = Path::new("/fpexif/raws/RAW_FUIJI_X-E2.RAF");
    if !test_path.exists() {
        println!("Skipping test - RAF file not found");
        return;
    }

    let parser = ExifParser::new();
    let result = parser.parse_file(test_path);

    match result {
        Ok(exif_data) => {
            if let Some(maker_notes) = exif_data.get_maker_notes() {
                // Look for DynamicRange (0x1400)
                if let Some(dr_tag) = maker_notes.get(&0x1400) {
                    println!("DynamicRange tag found: {:?}", dr_tag.value);

                    match &dr_tag.value {
                        ExifValue::Ascii(s) => {
                            println!("DynamicRange decoded: {}", s);
                            // Should be decoded (even if Unknown for unsupported cameras)
                            assert!(!s.is_empty(), "DynamicRange should be decoded");
                        }
                        ExifValue::Short(values) if !values.is_empty() => {
                            println!("DynamicRange raw value: {}", values[0]);
                            println!("Note: This value is not in our decoder - consider adding it");
                        }
                        _ => {
                            println!("DynamicRange has value: {:?}", dr_tag.value);
                        }
                    }
                } else {
                    println!("DynamicRange tag (0x1400) not found in this file");
                }
            }
        }
        Err(e) => {
            panic!("Failed to parse RAF file: {:?}", e);
        }
    }
}

#[test]
fn test_decode_sharpness() {
    let test_path = Path::new("/fpexif/raws/RAW_FUJI_E900.RAF");
    if !test_path.exists() {
        println!("Skipping test - RAF file not found");
        return;
    }

    let parser = ExifParser::new();
    let result = parser.parse_file(test_path);

    match result {
        Ok(exif_data) => {
            if let Some(maker_notes) = exif_data.get_maker_notes() {
                // Look for Sharpness (0x1001)
                if let Some(sharp_tag) = maker_notes.get(&0x1001) {
                    println!("Sharpness tag found: {:?}", sharp_tag.value);

                    match &sharp_tag.value {
                        ExifValue::Ascii(s) => {
                            println!("Sharpness decoded: {}", s);
                            assert!(!s.is_empty(), "Sharpness should be decoded");
                        }
                        _ => {
                            println!("Sharpness has value: {:?}", sharp_tag.value);
                        }
                    }
                } else {
                    println!("Sharpness tag (0x1001) not found in this file");
                }
            }
        }
        Err(e) => {
            panic!("Failed to parse RAF file: {:?}", e);
        }
    }
}

#[test]
fn test_decode_focus_mode() {
    let test_path = Path::new("/fpexif/raws/RAW_FUIJI_X-E2.RAF");
    if !test_path.exists() {
        println!("Skipping test - RAF file not found");
        return;
    }

    let parser = ExifParser::new();
    let result = parser.parse_file(test_path);

    match result {
        Ok(exif_data) => {
            if let Some(maker_notes) = exif_data.get_maker_notes() {
                // Look for FocusMode (0x1021)
                if let Some(focus_tag) = maker_notes.get(&0x1021) {
                    println!("FocusMode tag found: {:?}", focus_tag.value);

                    match &focus_tag.value {
                        ExifValue::Ascii(s) => {
                            println!("FocusMode decoded: {}", s);
                            assert!(
                                s.contains("Auto") || s.contains("Manual") || s.contains("AF-"),
                                "FocusMode should be a known value"
                            );
                        }
                        _ => {
                            println!("FocusMode has value: {:?}", focus_tag.value);
                        }
                    }
                } else {
                    println!("FocusMode tag (0x1021) not found in this file");
                }
            }
        }
        Err(e) => {
            panic!("Failed to parse RAF file: {:?}", e);
        }
    }
}

#[test]
fn test_decode_picture_mode() {
    let test_path = Path::new("/fpexif/raws/RAW_FUJI_F600XR.RAF");
    if !test_path.exists() {
        println!("Skipping test - RAF file not found");
        return;
    }

    let parser = ExifParser::new();
    let result = parser.parse_file(test_path);

    match result {
        Ok(exif_data) => {
            if let Some(maker_notes) = exif_data.get_maker_notes() {
                // Look for PictureMode (0x1031)
                if let Some(pic_tag) = maker_notes.get(&0x1031) {
                    println!("PictureMode tag found: {:?}", pic_tag.value);

                    match &pic_tag.value {
                        ExifValue::Ascii(s) => {
                            println!("PictureMode decoded: {}", s);
                            assert!(!s.is_empty(), "PictureMode should be decoded");
                        }
                        _ => {
                            println!("PictureMode has value: {:?}", pic_tag.value);
                        }
                    }
                } else {
                    println!("PictureMode tag (0x1031) not found in this file");
                }
            }
        }
        Err(e) => {
            panic!("Failed to parse RAF file: {:?}", e);
        }
    }
}

#[test]
fn test_decode_dynamic_range_setting() {
    let test_path = Path::new("/fpexif/raws/RAW_FUIJI_X-E2.RAF");
    if !test_path.exists() {
        println!("Skipping test - RAF file not found");
        return;
    }

    let parser = ExifParser::new();
    let result = parser.parse_file(test_path);

    match result {
        Ok(exif_data) => {
            if let Some(maker_notes) = exif_data.get_maker_notes() {
                // Look for DynamicRangeSetting (0x1402)
                if let Some(drs_tag) = maker_notes.get(&0x1402) {
                    println!("DynamicRangeSetting tag found: {:?}", drs_tag.value);

                    match &drs_tag.value {
                        ExifValue::Ascii(s) => {
                            println!("DynamicRangeSetting decoded: {}", s);
                            assert!(
                                s == "Auto" || s == "Manual",
                                "DynamicRangeSetting should be Auto or Manual"
                            );
                        }
                        _ => {
                            println!("DynamicRangeSetting has value: {:?}", drs_tag.value);
                        }
                    }
                } else {
                    println!("DynamicRangeSetting tag (0x1402) not found in this file");
                }
            }
        }
        Err(e) => {
            panic!("Failed to parse RAF file: {:?}", e);
        }
    }
}

#[test]
fn test_fuji_makernotes_exist() {
    // Test that we can parse any Fuji file and get maker notes
    let test_path = Path::new("/fpexif/raws/RAW_FUIJI_X-E2.RAF");
    if !test_path.exists() {
        println!("Skipping test - RAF file not found");
        return;
    }

    let parser = ExifParser::new();
    let result = parser.parse_file(test_path);

    match result {
        Ok(exif_data) => {
            let maker_notes = exif_data
                .get_maker_notes()
                .expect("Fuji RAF file should have maker notes");

            assert!(
                !maker_notes.is_empty(),
                "Should have parsed some maker note tags"
            );

            println!("Found {} maker note tags", maker_notes.len());

            // Print all tags for debugging
            for (tag_id, tag) in maker_notes {
                let name = tag.tag_name.unwrap_or("Unknown");
                println!("  Tag 0x{:04X} ({}): {:?}", tag_id, name, tag.value);
            }
        }
        Err(e) => {
            panic!("Failed to parse RAF file: {:?}", e);
        }
    }
}
