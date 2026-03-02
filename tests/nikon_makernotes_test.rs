// Nikon maker notes tests

use fpexif::ExifParser;
use fpexif::data_types::ExifValue;
use std::path::Path;

#[test]
fn test_nikon_d300_real_file() {
    let path = Path::new("/fpexif/raws/RAW_NIKON_D300.NEF");
    if !path.exists() {
        eprintln!("Test file not found: {:?}", path);
        return;
    }

    let parser = ExifParser::new();
    match parser.parse_file(path) {
        Ok(exif) => {
            // Check maker notes are present
            let maker_notes = exif
                .get_maker_notes()
                .expect("Nikon D300 should have maker notes");

            assert!(!maker_notes.is_empty(), "Should have Nikon maker notes");
            println!("Found {} maker note tags", maker_notes.len());

            // Print all maker note tags for debugging
            for (tag_id, tag) in maker_notes {
                if let Some(name) = tag.tag_name {
                    print!("Tag 0x{:04X} ({}): ", tag_id, name);
                    match &tag.value {
                        ExifValue::Ascii(s) => println!("{}", s),
                        ExifValue::Short(v) => println!("{:?}", v),
                        ExifValue::Long(v) => println!("{:?}", v),
                        ExifValue::Byte(v) => println!("{:?}", v),
                        ExifValue::Rational(v) => println!("{:?}", v),
                        _ => println!("{:?}", tag.value),
                    }
                }
            }

            // These are the specific decoded tags we added

            // Test ActiveDLighting decoding (if present)
            if let Some(tag) = maker_notes.get(&0x0022)
                && let ExifValue::Ascii(s) = &tag.value
            {
                println!("ActiveDLighting decoded: {}", s);
                // Valid values: Off, Low, Normal, High, Extra High, Auto, Unknown
                assert!(
                    s == "Off"
                        || s == "Low"
                        || s == "Normal"
                        || s == "High"
                        || s == "Extra High"
                        || s == "Auto"
                        || s == "Unknown"
                );
            }

            // Test ColorSpace decoding (if present)
            if let Some(tag) = maker_notes.get(&0x001E)
                && let ExifValue::Ascii(s) = &tag.value
            {
                println!("ColorSpace decoded: {}", s);
                // Valid values: sRGB, Adobe RGB, Unknown
                assert!(s == "sRGB" || s == "Adobe RGB" || s == "Unknown");
            }

            // Test VignetteControl decoding (if present)
            if let Some(tag) = maker_notes.get(&0x002A)
                && let ExifValue::Ascii(s) = &tag.value
            {
                println!("VignetteControl decoded: {}", s);
                // Valid values: Off, Low, Normal, High, Unknown
                assert!(s == "Off" || s == "Low" || s == "Normal" || s == "High" || s == "Unknown");
            }

            // Test LensType decoding (if present)
            if let Some(tag) = maker_notes.get(&0x0098)
                && let ExifValue::Ascii(s) = &tag.value
            {
                println!("LensType decoded: {}", s);
                // Should contain lens type features or "Unknown"
                assert!(!s.is_empty());
            }

            // Test Lens info decoding (if present)
            if let Some(tag) = maker_notes.get(&0x0099)
                && let ExifValue::Ascii(s) = &tag.value
            {
                println!("Lens decoded: {}", s);
                // Should be formatted like "28-70mm f/3.5-4.5" or similar
                assert!(s.contains("mm"));
            }
        }
        Err(e) => {
            panic!("Failed to parse Nikon D300 file: {:?}", e);
        }
    }
}

#[test]
fn test_nikon_d5300_real_file() {
    let path = Path::new("/fpexif/raws/RAW_NIKON_D5300.NEF");
    if !path.exists() {
        eprintln!("Test file not found: {:?}", path);
        return;
    }

    let parser = ExifParser::new();
    match parser.parse_file(path) {
        Ok(exif) => {
            let maker_notes = exif
                .get_maker_notes()
                .expect("Nikon D5300 should have maker notes");

            assert!(!maker_notes.is_empty(), "Should have Nikon maker notes");
            println!("Found {} maker note tags", maker_notes.len());

            // Check for decoded tags
            if let Some(tag) = maker_notes.get(&0x0022) {
                println!("ActiveDLighting: {:?}", tag.value);
            }

            if let Some(tag) = maker_notes.get(&0x001E) {
                println!("ColorSpace: {:?}", tag.value);
            }

            if let Some(tag) = maker_notes.get(&0x002A) {
                println!("VignetteControl: {:?}", tag.value);
            }
        }
        Err(e) => {
            panic!("Failed to parse Nikon D5300 file: {:?}", e);
        }
    }
}

#[test]
fn test_nikon_d3x_real_file() {
    let path = Path::new("/fpexif/raws/RAW_NIKON_D3X.NEF");
    if !path.exists() {
        eprintln!("Test file not found: {:?}", path);
        return;
    }

    let parser = ExifParser::new();
    match parser.parse_file(path) {
        Ok(exif) => {
            let maker_notes = exif
                .get_maker_notes()
                .expect("Nikon D3X should have maker notes");

            assert!(!maker_notes.is_empty(), "Should have Nikon maker notes");
            println!("Found {} maker note tags", maker_notes.len());

            // Check for lens info
            if let Some(tag) = maker_notes.get(&0x0099) {
                println!("Lens: {:?}", tag.value);
            }

            if let Some(tag) = maker_notes.get(&0x0098) {
                println!("LensType: {:?}", tag.value);
            }
        }
        Err(e) => {
            panic!("Failed to parse Nikon D3X file: {:?}", e);
        }
    }
}
