// makernotes/leaf.rs - Leaf/Creo maker notes parsing
//
// Reference files:
// - exiftool/lib/Image/ExifTool/Leaf.pm
//
// Leaf digital camera backs (Aptus series, etc.) use a proprietary PKTS-based
// format rather than standard EXIF IFD structure. The MakerNote is stored in
// EXIF tag 0x8606 and contains nested PKTS packets with string-based tags.
//
// Unlike other manufacturers, Leaf tags are hierarchical string names like:
// - "back_serial_number"
// - "CameraObj_ISO_speed"
// - "CameraObj_lens_type"
// - "SharpObj_sharp_info"
//
// exiv2 has no Leaf support.

use crate::data_types::{Endianness, ExifValue};
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use byteorder::{BigEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Cursor;

/// Leaf PKTS packet header size
const PKTS_HEADER_SIZE: usize = 52;

/// Get the name of a Leaf maker note tag
/// Since Leaf uses string-based tags, we return the tag name as-is
pub fn get_leaf_tag_name(tag_name: &str) -> String {
    // Map internal tag names to ExifTool-style names
    match tag_name {
        "back_serial_number" => "BackSerial".to_string(),
        "CaptProf_serial_number" => "CaptureSerial".to_string(),
        "CameraObj_ISO_speed" => "ISOSpeed".to_string(),
        "CameraObj_strobe" => "Strobe".to_string(),
        "CameraObj_camera_type" => "CameraType".to_string(),
        "CameraObj_lens_type" => "LensType".to_string(),
        "CameraObj_lens_ID" => "LensID".to_string(),
        "SharpObj_sharp_method" => "SharpMethod".to_string(),
        "SharpObj_data_len" => "DataLen".to_string(),
        "SharpObj_sharp_info" => "SharpInfo".to_string(),
        "ImgProf_image_status" => "ImageStatus".to_string(),
        "ImgProf_rotation_angle" => "RotationAngle".to_string(),
        "ColorObj_has_ICC" => "HasICC".to_string(),
        "ColorObj_input_profile" => "InputProfile".to_string(),
        "ColorObj_output_profile" => "OutputProfile".to_string(),
        "ColorObj_color_mode" => "ColorMode".to_string(),
        "ColorObj_color_type" => "ColorType".to_string(),
        _ => tag_name.to_string(),
    }
}

/// Decode Leaf camera type values
pub fn decode_camera_type(value: &str) -> &'static str {
    match value {
        "1" => "View Camera",
        "2" => "Medium Format",
        "3" => "Large Format",
        _ => "Unknown",
    }
}

/// Decode Leaf ISO speed values
pub fn decode_iso_speed(value: &str) -> &'static str {
    match value {
        "0" => "25",
        "1" => "32",
        "2" => "50",
        "3" => "64",
        "4" => "80",
        "5" => "100",
        "6" => "125",
        "7" => "160",
        "8" => "200",
        "9" => "250",
        "10" => "320",
        "11" => "400",
        "12" => "500",
        "13" => "640",
        "14" => "800",
        _ => "Unknown",
    }
}

/// Decode Leaf strobe/flash values
pub fn decode_strobe(value: &str) -> &'static str {
    match value {
        "0" => "Off",
        "1" => "On",
        _ => "Unknown",
    }
}

/// Parse a PKTS packet structure
/// Returns a HashMap of tag_name -> value
fn parse_pkts_packet(data: &[u8], offset: usize) -> Result<HashMap<String, String>, ExifError> {
    let mut result = HashMap::new();
    let mut pos = offset;

    while pos + PKTS_HEADER_SIZE <= data.len() {
        // Check for PKTS signature
        if &data[pos..pos + 4] != b"PKTS" {
            break;
        }

        // Read packet size from offset 48 (4 bytes, big-endian)
        let size = if pos + 52 <= data.len() {
            let mut cursor = Cursor::new(&data[pos + 48..pos + 52]);
            cursor.read_u32::<BigEndian>().unwrap_or(0) as usize
        } else {
            break;
        };

        // Extract tag name from offset 8 (40 bytes, null-terminated)
        let tag_name_bytes = &data[pos + 8..pos + 48];
        let tag_name = String::from_utf8_lossy(tag_name_bytes)
            .trim_end_matches('\0')
            .to_string();

        pos += PKTS_HEADER_SIZE;

        if pos + size > data.len() {
            break;
        }

        // Extract value
        if !tag_name.is_empty() {
            let value_bytes = &data[pos..pos + size];

            // Check if this is a nested PKTS structure
            if size >= 8 && &value_bytes[0..4] == b"PKTS" {
                // Recursively parse nested structure
                if let Ok(nested) = parse_pkts_packet(value_bytes, 0) {
                    // Merge nested tags with parent prefix
                    for (nested_tag, nested_value) in nested {
                        result.insert(nested_tag, nested_value);
                    }
                }
            } else {
                // Convert to string, removing null terminators
                let value = String::from_utf8_lossy(value_bytes)
                    .trim_end_matches('\0')
                    .trim()
                    .replace('\n', " ")
                    .to_string();

                if !value.is_empty() {
                    result.insert(tag_name, value);
                }
            }
        }

        pos += size;
    }

    Ok(result)
}

/// Parse Leaf maker notes
///
/// Leaf uses a PKTS-based hierarchical structure rather than IFD format.
/// We extract relevant tags and return them as MakerNoteTag entries.
pub fn parse_leaf_maker_notes(
    data: &[u8],
    _endian: Endianness,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    // Parse the PKTS structure
    let pkts_data = match parse_pkts_packet(data, 0) {
        Ok(d) => d,
        Err(_) => return Ok(tags),
    };

    // Assign synthetic tag IDs for common tags
    // We'll use a simple counter to generate unique IDs
    let mut tag_id: u16 = 0x0001;

    // Priority tags for MVP compliance
    let priority_tags = vec![
        "CaptProf_serial_number",
        "back_serial_number",
        "CameraObj_ISO_speed",
        "CameraObj_camera_type",
        "CameraObj_lens_type",
        "CameraObj_lens_ID",
        "CameraObj_strobe",
        "SharpObj_sharp_info",
        "ColorObj_input_profile",
        "ColorObj_output_profile",
        "ColorObj_color_mode",
        "ImgProf_rotation_angle",
    ];

    // Process priority tags first
    for tag_name in &priority_tags {
        if let Some(value_str) = pkts_data.get(*tag_name) {
            let display_name = get_leaf_tag_name(tag_name);

            // Decode specific tags
            let decoded_value = match *tag_name {
                "CameraObj_ISO_speed" => decode_iso_speed(value_str).to_string(),
                "CameraObj_camera_type" => decode_camera_type(value_str).to_string(),
                "CameraObj_strobe" => decode_strobe(value_str).to_string(),
                "CaptProf_serial_number" | "back_serial_number" => {
                    // ExifTool removes everything after first space
                    value_str
                        .split_whitespace()
                        .next()
                        .unwrap_or(value_str)
                        .to_string()
                }
                _ => value_str.to_string(),
            };

            // Store the display name in a box to get a static lifetime
            let name_box = Box::leak(display_name.into_boxed_str());

            tags.insert(
                tag_id,
                MakerNoteTag::new(tag_id, Some(name_box), ExifValue::Ascii(decoded_value)),
            );
            tag_id += 1;
        }
    }

    // Add other tags (non-priority) if they exist
    for (tag_name, value_str) in &pkts_data {
        if priority_tags.contains(&tag_name.as_str()) {
            continue; // Already processed
        }

        let display_name = get_leaf_tag_name(tag_name);
        let name_box = Box::leak(display_name.into_boxed_str());

        tags.insert(
            tag_id,
            MakerNoteTag::new(tag_id, Some(name_box), ExifValue::Ascii(value_str.clone())),
        );
        tag_id += 1;
    }

    Ok(tags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_camera_type() {
        assert_eq!(decode_camera_type("1"), "View Camera");
        assert_eq!(decode_camera_type("2"), "Medium Format");
        assert_eq!(decode_camera_type("3"), "Large Format");
        assert_eq!(decode_camera_type("99"), "Unknown");
    }

    #[test]
    fn test_decode_iso_speed() {
        assert_eq!(decode_iso_speed("0"), "25");
        assert_eq!(decode_iso_speed("2"), "50");
        assert_eq!(decode_iso_speed("5"), "100");
        assert_eq!(decode_iso_speed("11"), "400");
        assert_eq!(decode_iso_speed("99"), "Unknown");
    }

    #[test]
    fn test_decode_strobe() {
        assert_eq!(decode_strobe("0"), "Off");
        assert_eq!(decode_strobe("1"), "On");
        assert_eq!(decode_strobe("2"), "Unknown");
    }

    #[test]
    fn test_get_leaf_tag_name() {
        assert_eq!(
            get_leaf_tag_name("back_serial_number"),
            "BackSerial".to_string()
        );
        assert_eq!(
            get_leaf_tag_name("CaptProf_serial_number"),
            "CaptureSerial".to_string()
        );
        assert_eq!(
            get_leaf_tag_name("CameraObj_ISO_speed"),
            "ISOSpeed".to_string()
        );
        assert_eq!(get_leaf_tag_name("unknown_tag"), "unknown_tag".to_string());
    }
}
