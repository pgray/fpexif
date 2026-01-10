// Sony maker notes tests

use fpexif::data_types::ExifValue;
use fpexif::ExifParser;
use std::path::Path;

#[test]
fn test_sony_a7m2_value_decoding() {
    // Test with Sony ILCE-7M2 (A7 II)
    let test_file = Path::new("/fpexif/raws/RAW_SONY_ILCE-7M2.ARW");

    if !test_file.exists() {
        println!("Warning: Test file not found, skipping test");
        return;
    }

    let parser = ExifParser::new();
    let exif = parser
        .parse_file(test_file)
        .expect("Failed to parse Sony A7M2 file");

    // Check that maker notes were parsed
    if let Some(maker_notes) = exif.get_maker_notes() {
        println!("Found {} maker note tags", maker_notes.len());

        // Look for tags that should have decoded values
        for (tag_id, tag) in maker_notes {
            match *tag_id {
                0xB020 => {
                    // CreativeStyle
                    println!("CreativeStyle tag found: {:?}", tag.value);
                    // Can be Short or Ascii after decoding
                    match &tag.value {
                        ExifValue::Ascii(s) => println!("CreativeStyle decoded to: '{}'", s),
                        ExifValue::Short(v) => println!("CreativeStyle raw value: {:?}", v),
                        _ => println!("CreativeStyle unexpected type: {:?}", tag.value),
                    }
                }
                0xB041 => {
                    // ExposureMode
                    println!("ExposureMode tag found: {:?}", tag.value);
                    match &tag.value {
                        ExifValue::Ascii(s) => println!("ExposureMode decoded to: '{}'", s),
                        ExifValue::Short(v) => println!("ExposureMode raw value: {:?}", v),
                        _ => println!("ExposureMode unexpected type: {:?}", tag.value),
                    }
                }
                0xB043 => {
                    // AFMode
                    println!("AFMode tag found: {:?}", tag.value);
                    match &tag.value {
                        ExifValue::Ascii(s) => println!("AFMode decoded to: '{}'", s),
                        ExifValue::Short(v) => println!("AFMode raw value: {:?}", v),
                        _ => println!("AFMode unexpected type: {:?}", tag.value),
                    }
                }
                0xB025 => {
                    // DynamicRangeOptimizer
                    println!("DynamicRangeOptimizer tag found: {:?}", tag.value);
                    match &tag.value {
                        ExifValue::Ascii(s) => {
                            println!("DynamicRangeOptimizer decoded to: '{}'", s)
                        }
                        ExifValue::Short(v) => println!("DynamicRangeOptimizer raw value: {:?}", v),
                        ExifValue::Long(v) => {
                            println!("DynamicRangeOptimizer raw value (LONG): {:?}", v)
                        }
                        _ => println!("DynamicRangeOptimizer unexpected type: {:?}", tag.value),
                    }
                }
                0x201B => {
                    // FocusMode
                    println!("FocusMode tag found: {:?}", tag.value);
                    match &tag.value {
                        ExifValue::Ascii(s) => println!("FocusMode decoded to: '{}'", s),
                        ExifValue::Short(v) => println!("FocusMode raw value: {:?}", v),
                        _ => println!("FocusMode unexpected type: {:?}", tag.value),
                    }
                }
                0xB026 => {
                    // ImageStabilization
                    println!("ImageStabilization tag found: {:?}", tag.value);
                    match &tag.value {
                        ExifValue::Ascii(s) => println!("ImageStabilization decoded to: '{}'", s),
                        ExifValue::Short(v) => println!("ImageStabilization raw value: {:?}", v),
                        _ => println!("ImageStabilization unexpected type: {:?}", tag.value),
                    }
                }
                _ => {}
            }
        }
    } else {
        panic!("No maker notes found in Sony A7M2 file");
    }
}

#[test]
fn test_sony_a100_value_decoding() {
    // Test with Sony A100 (earlier A-mount camera)
    let test_file = Path::new("/fpexif/raws/RAW_SONY_A100.ARW");

    if !test_file.exists() {
        println!("Warning: Test file not found, skipping test");
        return;
    }

    let parser = ExifParser::new();
    let exif = parser
        .parse_file(test_file)
        .expect("Failed to parse Sony A100 file");

    // Check that maker notes were parsed
    if let Some(maker_notes) = exif.get_maker_notes() {
        println!("Found {} maker note tags", maker_notes.len());

        // Just verify we can parse the file and extract some maker notes
        assert!(
            !maker_notes.is_empty(),
            "Should have extracted some maker notes"
        );

        // Print any decoded values we find
        for (tag_id, tag) in maker_notes {
            if let Some(tag_name) = tag.tag_name {
                if let ExifValue::Ascii(s) = &tag.value {
                    println!("Tag 0x{:04X} ({}): {}", tag_id, tag_name, s);
                }
            }
        }
    } else {
        panic!("No maker notes found in Sony A100 file");
    }
}

#[test]
fn test_decode_creative_style_values() {
    // Test the decode_creative_style_exiftool function directly through parsing
    use fpexif::makernotes::sony::*;

    assert_eq!(decode_creative_style_exiftool(1), "Standard");
    assert_eq!(decode_creative_style_exiftool(2), "Vivid");
    assert_eq!(decode_creative_style_exiftool(3), "Portrait");
    assert_eq!(decode_creative_style_exiftool(4), "Landscape");
    assert_eq!(decode_creative_style_exiftool(8), "B&W");
    assert_eq!(decode_creative_style_exiftool(13), "Neutral");
    assert_eq!(decode_creative_style_exiftool(999), "Unknown");
}

#[test]
fn test_decode_focus_mode_values() {
    use fpexif::makernotes::sony::*;

    // Values from exiftool Sony.pm tag 0x201b
    assert_eq!(decode_focus_mode_exiftool(0), "Manual");
    assert_eq!(decode_focus_mode_exiftool(1), "Unknown"); // Value 1 not defined in exiftool
    assert_eq!(decode_focus_mode_exiftool(2), "AF-S");
    assert_eq!(decode_focus_mode_exiftool(3), "AF-C");
    assert_eq!(decode_focus_mode_exiftool(4), "AF-A");
    assert_eq!(decode_focus_mode_exiftool(6), "DMF");
    assert_eq!(decode_focus_mode_exiftool(7), "AF-D");
    assert_eq!(decode_focus_mode_exiftool(999), "Unknown");
}

#[test]
fn test_decode_af_mode_values() {
    use fpexif::makernotes::sony::*;

    assert_eq!(decode_af_mode_exiftool(0), "Default");
    assert_eq!(decode_af_mode_exiftool(1), "Multi");
    assert_eq!(decode_af_mode_exiftool(2), "Center");
    assert_eq!(decode_af_mode_exiftool(3), "Spot");
    assert_eq!(decode_af_mode_exiftool(4), "Flexible Spot");
    assert_eq!(decode_af_mode_exiftool(14), "Tracking");
    assert_eq!(decode_af_mode_exiftool(999), "Unknown");
}

#[test]
fn test_decode_exposure_mode_values() {
    use fpexif::makernotes::sony::*;

    assert_eq!(decode_exposure_mode_exiftool(0), "Program AE");
    assert_eq!(decode_exposure_mode_exiftool(1), "Portrait");
    assert_eq!(decode_exposure_mode_exiftool(2), "Beach");
    assert_eq!(decode_exposure_mode_exiftool(5), "Landscape");
    assert_eq!(decode_exposure_mode_exiftool(12), "Soft Snap/Portrait");
    assert_eq!(decode_exposure_mode_exiftool(999), "Unknown");
}

#[test]
fn test_decode_dynamic_range_optimizer_values() {
    use fpexif::makernotes::sony::*;

    assert_eq!(decode_dynamic_range_optimizer_exiftool(0), "Off");
    assert_eq!(decode_dynamic_range_optimizer_exiftool(1), "Standard");
    assert_eq!(decode_dynamic_range_optimizer_exiftool(2), "Advanced Auto");
    assert_eq!(decode_dynamic_range_optimizer_exiftool(3), "Auto");
    assert_eq!(decode_dynamic_range_optimizer_exiftool(8), "Advanced Lv5");
    assert_eq!(decode_dynamic_range_optimizer_exiftool(999), "Unknown");
}

#[test]
fn test_decode_image_stabilization_values() {
    use fpexif::makernotes::sony::*;

    assert_eq!(decode_image_stabilization_exiftool(0), "Off");
    assert_eq!(decode_image_stabilization_exiftool(1), "On");
    assert_eq!(decode_image_stabilization_exiftool(2), "On (2)");
    assert_eq!(decode_image_stabilization_exiftool(999), "Unknown");
}
