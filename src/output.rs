// output.rs - JSON output formatting for EXIF data

#[cfg(feature = "serde")]
use serde_json::{Map, Value};

#[cfg(feature = "serde")]
use crate::data_types::ExifValue;
#[cfg(feature = "serde")]
use crate::ExifData;

/// Calculate greatest common divisor
fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// Format a single Short value with tag-specific interpretation
#[cfg(feature = "serde")]
fn format_short_value(value: u16, tag_id: u16) -> Value {
    match tag_id {
        0x0106 => Value::String(
            crate::tags::get_photometric_interpretation_description(value).to_string(),
        ),
        0x0112 => Value::String(crate::tags::get_orientation_description(value).to_string()),
        0x0103 => Value::String(crate::tags::get_compression_description(value).to_string()),
        0x011C => {
            Value::String(crate::tags::get_planar_configuration_description(value).to_string())
        }
        0x0128 => Value::String(crate::tags::get_resolution_unit_description(value).to_string()),
        0x0213 => Value::String(crate::tags::get_ycbcr_positioning_description(value).to_string()),
        0x8822 => Value::String(crate::tags::get_exposure_program_description(value).to_string()),
        0x9207 => Value::String(crate::tags::get_metering_mode_description(value).to_string()),
        0x9208 => Value::String(crate::tags::get_light_source_description(value).to_string()),
        0x9209 => Value::String(crate::tags::get_flash_description(value).to_string()),
        0xA001 => Value::String(crate::tags::get_color_space_description(value).to_string()),
        0xA210 => Value::String(
            match value {
                2 => "inches",
                3 => "cm",
                _ => "unknown",
            }
            .to_string(),
        ),
        0xA402 => Value::String(crate::tags::get_exposure_mode_description(value).to_string()),
        0xA403 => Value::String(crate::tags::get_white_balance_description(value).to_string()),
        0xA406 => Value::String(crate::tags::get_scene_capture_type_description(value).to_string()),
        0xA407 => Value::String(crate::tags::get_gain_control_description(value).to_string()),
        // Contrast, Saturation, Sharpness - exiftool outputs raw numeric values
        0xA408..=0xA40A => Value::Number(value.into()),
        0xA40C => {
            Value::String(crate::tags::get_subject_distance_range_description(value).to_string())
        }
        0xA401 => Value::String(crate::tags::get_custom_rendered_description(value).to_string()),
        0xA217 => Value::String(crate::tags::get_sensing_method_description(value).to_string()),
        0x8830 => Value::String(crate::tags::get_sensitivity_type_description(value).to_string()),
        // DNG CalibrationIlluminant tags use LightSource descriptions
        0xC65A | 0xC65B => {
            Value::String(crate::tags::get_light_source_description(value).to_string())
        }
        _ => Value::Number(value.into()),
    }
}

/// Format a single Rational value with tag-specific interpretation
#[cfg(feature = "serde")]
fn format_rational_value(num: u32, den: u32, tag_id: u16) -> Value {
    if den == 0 {
        return Value::String("inf".to_string());
    }

    match tag_id {
        0x829A => {
            // ExposureTime - show as simplified fraction
            let gcd_val = gcd(num, den);
            let simplified_num = num / gcd_val;
            let simplified_den = den / gcd_val;
            if simplified_num == 1 {
                Value::String(format!("1/{}", simplified_den))
            } else {
                Value::String(format!("{}/{}", simplified_num, simplified_den))
            }
        }
        0x9201 => {
            // ShutterSpeedValue (APEX) - convert to shutter speed fraction
            let apex_value = num as f64 / den as f64;
            let shutter_speed = 2f64.powf(apex_value);
            let denominator = shutter_speed.round() as u32;
            Value::String(format!("1/{}", denominator))
        }
        0x9202 | 0x9205 => {
            // ApertureValue, MaxApertureValue (APEX) - convert to f-number
            let apex_value = num as f64 / den as f64;
            let f_number = 2f64.powf(apex_value / 2.0);
            let rounded = (f_number * 10.0).round() / 10.0;
            serde_json::Number::from_f64(rounded)
                .map(Value::Number)
                .unwrap_or_else(|| Value::String(rounded.to_string()))
        }
        0x920A => {
            // FocalLength - add mm unit with decimal formatting
            let focal_length = num as f64 / den as f64;
            // Format with at least one decimal place for exiftool compatibility
            if focal_length.fract() == 0.0 {
                Value::String(format!("{:.1} mm", focal_length))
            } else {
                Value::String(format!("{} mm", focal_length))
            }
        }
        0x9203 | 0x9204 => {
            // Other APEX values - show as decimal
            let decimal = num as f64 / den as f64;
            serde_json::Number::from_f64(decimal)
                .map(Value::Number)
                .unwrap_or_else(|| Value::String(decimal.to_string()))
        }
        _ => {
            // Default: show as decimal
            let decimal = num as f64 / den as f64;
            serde_json::Number::from_f64(decimal)
                .map(Value::Number)
                .unwrap_or_else(|| Value::String(decimal.to_string()))
        }
    }
}

/// Format a single Signed Rational value with tag-specific interpretation
#[cfg(feature = "serde")]
fn format_srational_value(num: i32, den: i32, tag_id: u16) -> Value {
    if den == 0 {
        return Value::String("inf".to_string());
    }

    match tag_id {
        0x9201 => {
            // ShutterSpeedValue (APEX) - convert to shutter speed fraction
            let apex_value = num as f64 / den as f64;
            let shutter_speed = 2f64.powf(apex_value);
            let denominator = shutter_speed.round() as i32;
            Value::String(format!("1/{}", denominator))
        }
        _ => {
            let decimal = num as f64 / den as f64;
            serde_json::Number::from_f64(decimal)
                .map(Value::Number)
                .unwrap_or_else(|| Value::String(decimal.to_string()))
        }
    }
}

/// Simple base64 encoding
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let b1 = chunk[0];
        let b2 = chunk.get(1).copied().unwrap_or(0);
        let b3 = chunk.get(2).copied().unwrap_or(0);

        result.push(CHARS[(b1 >> 2) as usize] as char);
        result.push(CHARS[(((b1 & 0x03) << 4) | (b2 >> 4)) as usize] as char);
        result.push(if chunk.len() > 1 {
            CHARS[(((b2 & 0x0f) << 2) | (b3 >> 6)) as usize] as char
        } else {
            '='
        });
        result.push(if chunk.len() > 2 {
            CHARS[(b3 & 0x3f) as usize] as char
        } else {
            '='
        });
    }

    result
}

/// Format Undefined type data with tag-specific interpretation
#[cfg(feature = "serde")]
fn format_undefined_value(data: &[u8], tag_id: u16) -> Value {
    match tag_id {
        0x9000 | 0xA000 => {
            // ExifVersion, FlashpixVersion - decode ASCII bytes to string
            String::from_utf8(data.to_vec())
                .map(Value::String)
                .unwrap_or_else(|_| {
                    Value::String(
                        data.iter()
                            .map(|b| format!("{:02x}", b))
                            .collect::<Vec<_>>()
                            .join(" "),
                    )
                })
        }
        0x9101 if data.len() == 4 => {
            // ComponentsConfiguration - decode to channel names
            let channels: Vec<&str> = data
                .iter()
                .map(|&b| match b {
                    0 => "-",
                    1 => "Y",
                    2 => "Cb",
                    3 => "Cr",
                    4 => "R",
                    5 => "G",
                    6 => "B",
                    _ => "?",
                })
                .collect();
            Value::String(channels.join(", "))
        }
        0x9286 => {
            // UserComment - check for charset marker and handle null padding
            if data.len() >= 8 {
                let charset = &data[0..8];
                let content = &data[8..];

                // Check for ASCII charset marker
                if charset == b"ASCII\0\0\0" {
                    // Check if content is just nulls/spaces (empty comment)
                    let cleaned: String = content
                        .iter()
                        .filter(|&&b| b != 0)
                        .map(|&b| b as char)
                        .collect::<String>()
                        .trim()
                        .to_string();
                    return Value::String(cleaned);
                }

                // Check for Unicode charset marker
                if charset == b"UNICODE\0" {
                    // Try to decode as UTF-16
                    if content.len() >= 2 {
                        // Check for BOM
                        let is_le = content.len() >= 2 && content[0] == 0xFF && content[1] == 0xFE;
                        let is_be = content.len() >= 2 && content[0] == 0xFE && content[1] == 0xFF;
                        let start = if is_le || is_be { 2 } else { 0 };

                        if content.len() > start {
                            let u16_values: Vec<u16> = content[start..]
                                .chunks(2)
                                .filter_map(|chunk| {
                                    if chunk.len() == 2 {
                                        Some(if is_be {
                                            u16::from_be_bytes([chunk[0], chunk[1]])
                                        } else {
                                            u16::from_le_bytes([chunk[0], chunk[1]])
                                        })
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            if let Ok(s) = String::from_utf16(&u16_values) {
                                let cleaned = s.trim_end_matches('\0').trim().to_string();
                                return Value::String(cleaned);
                            }
                        }
                    }
                }

                // Check if content after charset marker is all nulls
                if content.iter().all(|&b| b == 0) {
                    return Value::String(String::new());
                }
            }

            // Check if entire data is nulls (empty comment without charset marker)
            if data.iter().all(|&b| b == 0) {
                return Value::String(String::new());
            }

            // Default: return as base64 for non-empty binary data
            if data.len() <= 32 {
                Value::String(
                    data.iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" "),
                )
            } else {
                Value::String(base64_encode(data))
            }
        }
        0xA300 if data.len() == 1 => {
            // FileSource
            Value::String(crate::tags::get_file_source_description(data[0]).to_string())
        }
        0xA301 if data.len() == 1 => {
            // SceneType
            Value::String(crate::tags::get_scene_type_description(data[0]).to_string())
        }
        // CFAPattern in EXIF SubIFD (0xA302) as Undefined type
        0xA302 if data.len() >= 4 => {
            if let Some(formatted) = format_cfa_pattern(data) {
                Value::String(formatted)
            } else {
                Value::String(base64_encode(data))
            }
        }
        _ => {
            if data.len() <= 32 {
                // For short undefined data, show as hex
                Value::String(
                    data.iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" "),
                )
            } else {
                // For longer data, use base64
                Value::String(base64_encode(data))
            }
        }
    }
}

/// Tags that should be formatted as space-separated strings instead of JSON arrays
#[cfg(feature = "serde")]
fn should_format_as_space_separated(tag_id: u16) -> bool {
    matches!(
        tag_id,
        0x0102 // BitsPerSample
        | 0x0013 // ThumbnailImageValidArea (Canon)
        | 0x828D // CFARepeatPatternDim
        | 0x0214 // ReferenceBlackWhite
        | 0xC620 // DefaultCropSize (DNG)
    )
}

/// Tags that should be formatted as space-separated decimals for rational/srational arrays
#[cfg(feature = "serde")]
fn should_format_rationals_as_space_separated(tag_id: u16) -> bool {
    matches!(
        tag_id,
        0xC621 // ColorMatrix1 (DNG)
        | 0xC622 // ColorMatrix2 (DNG)
        | 0xC627 // AnalogBalance (DNG)
        | 0xC628 // AsShotNeutral (DNG)
    )
}

/// Format CFA pattern bytes as ExifTool-compatible string
/// Converts [0,1,1,2] to "[Red,Green][Green,Blue]"
#[cfg(feature = "serde")]
fn format_cfa_pattern(data: &[u8]) -> Option<String> {
    // CFA pattern for 2x2 Bayer pattern should have 4 bytes
    if data.len() < 4 {
        return None;
    }

    let color_name = |c: u8| -> &'static str {
        match c {
            0 => "Red",
            1 => "Green",
            2 => "Blue",
            3 => "Cyan",
            4 => "Magenta",
            5 => "Yellow",
            6 => "White",
            _ => "Unknown",
        }
    };

    // Format as [Row1][Row2] for 2x2 pattern
    Some(format!(
        "[{},{}][{},{}]",
        color_name(data[0]),
        color_name(data[1]),
        color_name(data[2]),
        color_name(data[3])
    ))
}

/// Convert an ExifValue to a JSON value in exiftool-compatible format
#[cfg(feature = "serde")]
pub fn format_exif_value_for_json(value: &ExifValue, tag_id: u16) -> Value {
    match value {
        // ASCII strings - return as plain string
        ExifValue::Ascii(s) => {
            let cleaned = s.trim_end_matches('\0').trim();
            Value::String(cleaned.to_string())
        }

        // Single-value Byte
        ExifValue::Byte(v) if v.len() == 1 => match tag_id {
            0xA300 => Value::String(crate::tags::get_file_source_description(v[0]).to_string()),
            0xA301 => Value::String(crate::tags::get_scene_type_description(v[0]).to_string()),
            // GPSAltitudeRef (0x0005 in GPS IFD)
            0x0005 => {
                Value::String(crate::tags::get_gps_altitude_ref_description(v[0]).to_string())
            }
            _ => Value::Number(v[0].into()),
        },

        // Single-value Short - use helper function
        ExifValue::Short(v) if v.len() == 1 => format_short_value(v[0], tag_id),

        // Single-value other numeric types
        ExifValue::Long(v) if v.len() == 1 => Value::Number(v[0].into()),
        ExifValue::SByte(v) if v.len() == 1 => Value::Number(v[0].into()),
        ExifValue::SShort(v) if v.len() == 1 => Value::Number(v[0].into()),
        ExifValue::SLong(v) if v.len() == 1 => Value::Number(v[0].into()),

        // Single-value Float/Double
        ExifValue::Float(v) if v.len() == 1 => serde_json::Number::from_f64(v[0] as f64)
            .map(Value::Number)
            .unwrap_or_else(|| Value::String(v[0].to_string())),
        ExifValue::Double(v) if v.len() == 1 => serde_json::Number::from_f64(v[0])
            .map(Value::Number)
            .unwrap_or_else(|| Value::String(v[0].to_string())),

        // Single-value Rational - use helper function
        ExifValue::Rational(v) if v.len() == 1 => {
            let (num, den) = v[0];
            format_rational_value(num, den, tag_id)
        }

        // Single-value Signed Rational - use helper function
        ExifValue::SRational(v) if v.len() == 1 => {
            let (num, den) = v[0];
            format_srational_value(num, den, tag_id)
        }

        // Multi-value arrays
        ExifValue::Byte(v) => {
            // GPSVersionID (0x0000), DNGVersion (0xC612), DNGBackwardVersion (0xC613) should be formatted as "2.2.0.0"
            if (tag_id == 0x0000 || tag_id == 0xC612 || tag_id == 0xC613) && v.len() == 4 {
                Value::String(format!("{}.{}.{}.{}", v[0], v[1], v[2], v[3]))
            // CFAPattern (0x828E TIFF/EP, 0xA302 EXIF) - format as [Red,Green][Green,Blue]
            } else if (tag_id == 0x828E || tag_id == 0xA302) && v.len() >= 4 {
                if let Some(formatted) = format_cfa_pattern(v) {
                    Value::String(formatted)
                } else {
                    Value::Array(v.iter().map(|&n| Value::Number(n.into())).collect())
                }
            } else if should_format_as_space_separated(tag_id) {
                Value::String(
                    v.iter()
                        .map(|n| n.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                )
            } else {
                Value::Array(v.iter().map(|&n| Value::Number(n.into())).collect())
            }
        }
        ExifValue::Short(v) => {
            if should_format_as_space_separated(tag_id) {
                Value::String(
                    v.iter()
                        .map(|n| n.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                )
            } else {
                Value::Array(v.iter().map(|&n| Value::Number(n.into())).collect())
            }
        }
        ExifValue::Long(v) => {
            if should_format_as_space_separated(tag_id) {
                Value::String(
                    v.iter()
                        .map(|n| n.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                )
            } else {
                Value::Array(v.iter().map(|&n| Value::Number(n.into())).collect())
            }
        }
        ExifValue::SByte(v) => Value::Array(v.iter().map(|&n| Value::Number(n.into())).collect()),
        ExifValue::SShort(v) => {
            if should_format_as_space_separated(tag_id) {
                Value::String(
                    v.iter()
                        .map(|n| n.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                )
            } else {
                Value::Array(v.iter().map(|&n| Value::Number(n.into())).collect())
            }
        }
        ExifValue::SLong(v) => Value::Array(v.iter().map(|&n| Value::Number(n.into())).collect()),

        // Multi-value Float/Double
        ExifValue::Float(v) => Value::Array(
            v.iter()
                .map(|&f| {
                    serde_json::Number::from_f64(f as f64)
                        .map(Value::Number)
                        .unwrap_or_else(|| Value::String(f.to_string()))
                })
                .collect(),
        ),
        ExifValue::Double(v) => Value::Array(
            v.iter()
                .map(|&f| {
                    serde_json::Number::from_f64(f)
                        .map(Value::Number)
                        .unwrap_or_else(|| Value::String(f.to_string()))
                })
                .collect(),
        ),

        // Multi-value Rationals
        ExifValue::Rational(v) => {
            if should_format_rationals_as_space_separated(tag_id) {
                // Format as space-separated decimals for DNG color matrix tags
                let decimals: Vec<String> = v
                    .iter()
                    .map(|&(num, den)| {
                        if den == 0 {
                            "inf".to_string()
                        } else {
                            let decimal = num as f64 / den as f64;
                            decimal.to_string()
                        }
                    })
                    .collect();
                Value::String(decimals.join(" "))
            } else {
                Value::Array(
                    v.iter()
                        .map(|&(num, den)| {
                            if den == 0 {
                                Value::String("inf".to_string())
                            } else {
                                let decimal = num as f64 / den as f64;
                                serde_json::Number::from_f64(decimal)
                                    .map(Value::Number)
                                    .unwrap_or_else(|| Value::String(decimal.to_string()))
                            }
                        })
                        .collect(),
                )
            }
        }
        ExifValue::SRational(v) => {
            if should_format_rationals_as_space_separated(tag_id) {
                // Format as space-separated decimals for DNG color matrix tags
                let decimals: Vec<String> = v
                    .iter()
                    .map(|&(num, den)| {
                        if den == 0 {
                            "inf".to_string()
                        } else {
                            let decimal = num as f64 / den as f64;
                            decimal.to_string()
                        }
                    })
                    .collect();
                Value::String(decimals.join(" "))
            } else {
                Value::Array(
                    v.iter()
                        .map(|&(num, den)| {
                            if den == 0 {
                                Value::String("inf".to_string())
                            } else {
                                let decimal = num as f64 / den as f64;
                                serde_json::Number::from_f64(decimal)
                                    .map(Value::Number)
                                    .unwrap_or_else(|| Value::String(decimal.to_string()))
                            }
                        })
                        .collect(),
                )
            }
        }

        // Undefined - use helper function
        ExifValue::Undefined(v) => format_undefined_value(v, tag_id),
    }
}

/// Convert ExifData to exiftool-compatible JSON
#[cfg(feature = "serde")]
pub fn to_exiftool_json(exif_data: &ExifData, source_file: Option<&str>) -> Value {
    let mut output = Map::new();

    // Add SourceFile field if provided
    if let Some(file) = source_file {
        output.insert("SourceFile".to_string(), Value::String(file.to_string()));
    }

    // Convert each tag to a key-value pair
    for (tag_id, value) in exif_data.iter() {
        let tag_name = if let Some(name) = tag_id.name() {
            name.to_string()
        } else {
            format!("Tag{}", tag_id.id)
        };
        let json_value = format_exif_value_for_json(value, tag_id.id);
        output.insert(tag_name, json_value);
    }

    // Add derived fields for exiftool compatibility
    // Aperture is derived from FNumber
    if let Some(ExifValue::Rational(v)) = exif_data.get_tag_by_id(0x829D) {
        if !v.is_empty() && v[0].1 != 0 {
            let aperture = v[0].0 as f64 / v[0].1 as f64;
            if let Some(num) = serde_json::Number::from_f64(aperture) {
                output.insert("Aperture".to_string(), Value::Number(num));
            }
        }
    }

    // ISO is an alias for ISOSpeedRatings (tag 0x8827)
    if let Some(ExifValue::Short(v)) = exif_data.get_tag_by_id(0x8827) {
        if !v.is_empty() {
            output.insert("ISO".to_string(), Value::Number(v[0].into()));
        }
    }

    // Add maker notes if present
    if let Some(maker_notes) = exif_data.get_maker_notes() {
        for (tag_id, maker_tag) in maker_notes.iter() {
            let tag_name = maker_tag
                .tag_name
                .unwrap_or_else(|| Box::leak(format!("MakerNote{:04X}", tag_id).into_boxed_str()));
            let json_value = format_exif_value_for_json(&maker_tag.value, *tag_id);
            output.insert(tag_name.to_string(), json_value);
        }
    }

    // Wrap in an array like exiftool does
    Value::Array(vec![Value::Object(output)])
}
