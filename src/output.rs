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
        0x0112 => Value::String(crate::tags::get_orientation_description(value).to_string()),
        0x0103 => Value::String(crate::tags::get_compression_description(value).to_string()),
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
        0xA408 => Value::String(crate::tags::get_contrast_description(value).to_string()),
        0xA409 => Value::String(crate::tags::get_saturation_description(value).to_string()),
        0xA40A => Value::String(crate::tags::get_sharpness_description(value).to_string()),
        0xA40C => {
            Value::String(crate::tags::get_subject_distance_range_description(value).to_string())
        }
        0xA401 => Value::String(crate::tags::get_custom_rendered_description(value).to_string()),
        0xA40B => Value::String(crate::tags::get_gain_control_description(value).to_string()),
        0x041A => Value::String(crate::tags::get_sensing_method_description(value).to_string()),
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
            // FocalLength - add mm unit
            let focal_length = num as f64 / den as f64;
            Value::String(format!("{} mm", focal_length))
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
        0xA300 if data.len() == 1 => {
            // FileSource
            Value::String(crate::tags::get_file_source_description(data[0]).to_string())
        }
        0xA301 if data.len() == 1 => {
            // SceneType
            Value::String(crate::tags::get_scene_type_description(data[0]).to_string())
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
        ExifValue::Byte(v) => Value::Array(v.iter().map(|&n| Value::Number(n.into())).collect()),
        ExifValue::Short(v) => Value::Array(v.iter().map(|&n| Value::Number(n.into())).collect()),
        ExifValue::Long(v) => Value::Array(v.iter().map(|&n| Value::Number(n.into())).collect()),
        ExifValue::SByte(v) => Value::Array(v.iter().map(|&n| Value::Number(n.into())).collect()),
        ExifValue::SShort(v) => Value::Array(v.iter().map(|&n| Value::Number(n.into())).collect()),
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
        ExifValue::Rational(v) => Value::Array(
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
        ),
        ExifValue::SRational(v) => Value::Array(
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
        ),

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
