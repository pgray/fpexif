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

/// Format EXIF MeteringMode - use manufacturer-specific naming where appropriate
#[cfg(feature = "serde")]
fn format_metering_mode(value: u16, make: Option<&str>) -> String {
    // Olympus uses "ESP" (Electro-Selective Pattern) instead of "Multi-segment"
    if let Some(m) = make {
        if m.to_uppercase().contains("OLYMPUS") || m.to_uppercase().contains("OM DIGITAL") {
            return match value {
                0 => "Unknown",
                1 => "Average",
                2 => "Center-weighted average",
                3 => "Spot",
                4 => "Multi-spot",
                5 => "ESP",
                6 => "Partial",
                255 => "Other",
                _ => "Unknown",
            }
            .to_string();
        }
    }
    crate::tags::get_metering_mode_description(value).to_string()
}

/// Format a single Short value with tag-specific interpretation and optional manufacturer info
#[cfg(feature = "serde")]
fn format_short_value_with_make(value: u16, tag_id: u16, make: Option<&str>) -> Value {
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
        0x9207 => Value::String(format_metering_mode(value, make)),
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
        // Contrast (0xA408), Saturation (0xA409), Sharpness (0xA40A)
        // Use text descriptions for all manufacturers
        0xA408 => Value::String(crate::tags::get_contrast_description(value).to_string()),
        0xA409 => Value::String(crate::tags::get_saturation_description(value).to_string()),
        0xA40A => Value::String(crate::tags::get_sharpness_description(value).to_string()),
        0xA40C => {
            Value::String(crate::tags::get_subject_distance_range_description(value).to_string())
        }
        0xA401 => Value::String(crate::tags::get_custom_rendered_description(value).to_string()),
        0x9217 | 0xA217 => {
            Value::String(crate::tags::get_sensing_method_description(value).to_string())
        }
        0x8830 => Value::String(crate::tags::get_sensitivity_type_description(value).to_string()),
        // DNG CalibrationIlluminant tags use LightSource descriptions
        0xC65A | 0xC65B => {
            Value::String(crate::tags::get_light_source_description(value).to_string())
        }
        // FocalLengthIn35mmFormat - add "mm" suffix
        0xA405 => Value::String(format!("{} mm", value)),
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
            // ExifTool approximates to 1/n form for readability
            let gcd_val = gcd(num, den);
            let simplified_num = num / gcd_val;
            let simplified_den = den / gcd_val;
            if simplified_num == 1 && simplified_den == 1 {
                // For 1 second exposure, ExifTool outputs just "1"
                Value::Number(1.into())
            } else if simplified_num == 1 {
                Value::String(format!("1/{}", simplified_den))
            } else {
                // Approximate to 1/n form: calculate equivalent denominator
                let exposure_time = num as f64 / den as f64;
                let approx_den = (1.0 / exposure_time).round() as u32;
                if approx_den == 1 {
                    Value::Number(1.into())
                } else {
                    Value::String(format!("1/{}", approx_den))
                }
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
        0x829D => {
            // FNumber - format with at least one decimal place for ExifTool compatibility
            let f_number = num as f64 / den as f64;
            let rounded = (f_number * 10.0).round() / 10.0;
            // ExifTool always shows one decimal place (e.g., 8.0, not 8)
            Value::Number(
                serde_json::Number::from_f64(rounded)
                    .unwrap_or_else(|| serde_json::Number::from(0)),
            )
        }
        0x920A => {
            // FocalLength - add mm unit with decimal formatting
            // ExifTool shows "55.0 mm" for CR2/standard EXIF
            // CRW shows "400 mm" but that comes from MakerNote, not EXIF
            let focal_length = num as f64 / den as f64;
            Value::String(format!("{:.1} mm", focal_length))
        }
        0x9203 => {
            // BrightnessValue - show as decimal number
            let decimal = num as f64 / den as f64;
            serde_json::Number::from_f64(decimal)
                .map(Value::Number)
                .unwrap_or_else(|| Value::String(decimal.to_string()))
        }
        0x9204 => {
            // ExposureBiasValue/ExposureCompensation (unsigned version - rare)
            // ExifTool outputs fractions when denominator simplifies to 3 or 2
            let decimal = num as f64 / den as f64;
            if decimal == 0.0 {
                Value::Number(0.into())
            } else {
                // Simplify the fraction (unsigned values are always positive)
                let gcd_val = gcd(num, den);
                let simple_num = num / gcd_val;
                let simple_den = den / gcd_val;

                if simple_den == 3 || simple_den == 2 {
                    // Output as fraction (e.g., +1/3, +2/3, +1/2)
                    Value::String(format!("+{}/{}", simple_num, simple_den))
                } else if decimal.fract() == 0.0 {
                    // Whole number (e.g., +1, +2)
                    let int_val = decimal as i32;
                    Value::String(format!("+{}", int_val))
                } else {
                    // Decimal format (e.g., +0.33, +0.67)
                    let rounded = (decimal * 100.0).round() / 100.0;
                    let formatted = format!("+{:.2}", rounded)
                        .trim_end_matches('0')
                        .trim_end_matches('.')
                        .to_string();
                    Value::String(formatted)
                }
            }
        }
        0x9206 => {
            // SubjectDistance - add " m" suffix
            let distance = num as f64 / den as f64;
            Value::String(format!("{} m", distance))
        }
        // Nikon MakerNote tags that should always show decimal format as JSON number
        0x008B => {
            // LensFStops - ExifTool outputs with exactly 2 decimal places (e.g., 6.00)
            // Use serde_json RawValue to preserve the exact formatting
            let decimal = num as f64 / den as f64;
            let formatted = format!("{:.2}", decimal);
            // Parse back as a number to get the correct JSON representation
            serde_json::from_str(&formatted).unwrap_or_else(|_| Value::Number(0.into()))
        }
        0x0017 => {
            // FlashExposureBracketValue
            // Return as JSON number to match ExifTool's output
            let decimal = num as f64 / den as f64;
            serde_json::Number::from_f64(decimal)
                .map(Value::Number)
                .unwrap_or_else(|| Value::Number(0.into()))
        }
        0xA20E | 0xA20F => {
            // FocalPlaneXResolution, FocalPlaneYResolution - round to 6 decimal places
            let decimal = num as f64 / den as f64;
            let rounded = (decimal * 1_000_000.0).round() / 1_000_000.0;
            serde_json::Number::from_f64(rounded)
                .map(Value::Number)
                .unwrap_or_else(|| Value::String(rounded.to_string()))
        }
        _ => {
            // Default: show as decimal, strip .0 for whole numbers to match ExifTool
            let decimal = num as f64 / den as f64;
            if decimal.fract() == 0.0 && decimal.is_finite() {
                // Whole number - output as integer JSON number
                let int_value = decimal as i64;
                Value::Number(int_value.into())
            } else {
                // Decimal number
                serde_json::Number::from_f64(decimal)
                    .map(Value::Number)
                    .unwrap_or_else(|| Value::String(decimal.to_string()))
            }
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
        0x9204 => {
            // ExposureBiasValue/ExposureCompensation
            // ExifTool outputs fractions when denominator simplifies to 3 or 2
            let decimal = num as f64 / den as f64;
            if decimal == 0.0 {
                Value::Number(0.into())
            } else {
                // Simplify the fraction
                let gcd_val = gcd(num.unsigned_abs(), den.unsigned_abs());
                let simple_num = num.unsigned_abs() / gcd_val;
                let simple_den = den.unsigned_abs() / gcd_val;
                let sign = if decimal > 0.0 { "+" } else { "-" };

                if simple_den == 3 || simple_den == 2 {
                    // Output as fraction (e.g., +1/3, -2/3, +1/2)
                    Value::String(format!("{}{}/{}", sign, simple_num, simple_den))
                } else if decimal.fract() == 0.0 {
                    // Whole number (e.g., +1, -2)
                    let int_val = decimal.abs() as i32;
                    Value::String(format!("{}{}", sign, int_val))
                } else {
                    // Decimal format (e.g., +0.33, -0.67)
                    let rounded = (decimal * 100.0).round() / 100.0;
                    let formatted = if rounded > 0.0 {
                        format!("+{:.2}", rounded)
                            .trim_end_matches('0')
                            .trim_end_matches('.')
                            .to_string()
                    } else {
                        format!("{:.2}", rounded)
                            .trim_end_matches('0')
                            .trim_end_matches('.')
                            .to_string()
                    };
                    Value::String(formatted)
                }
            }
        }
        // Nikon MakerNote tags that should always show decimal format
        0x0017 => {
            // FlashExposureBracketValue (0x0017) - always show decimal format as JSON number
            let decimal = num as f64 / den as f64;
            // Return as JSON number to match ExifTool's output
            serde_json::Number::from_f64(decimal)
                .map(Value::Number)
                .unwrap_or_else(|| Value::Number(0.into()))
        }
        _ => {
            // Default: show as decimal, strip .0 for whole numbers to match ExifTool
            let decimal = num as f64 / den as f64;
            if decimal.fract() == 0.0 && decimal.is_finite() {
                // Whole number - output as integer JSON number
                let int_value = decimal as i64;
                Value::Number(int_value.into())
            } else {
                // Decimal number
                serde_json::Number::from_f64(decimal)
                    .map(Value::Number)
                    .unwrap_or_else(|| Value::String(decimal.to_string()))
            }
        }
    }
}

/// Format binary data in ExifTool's style
/// Returns "(Binary data X bytes, use -b option to extract)"
fn format_binary_data(data: &[u8]) -> String {
    format!(
        "(Binary data {} bytes, use -b option to extract)",
        data.len()
    )
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

            // Check if entire data is nulls or spaces (empty comment without charset marker)
            if data.iter().all(|&b| b == 0 || b == 0x20) {
                return Value::String(String::new());
            }

            // Default: return as binary data message for non-empty binary data
            if data.len() <= 32 {
                Value::String(
                    data.iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" "),
                )
            } else {
                Value::String(format_binary_data(data))
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
        // CFAPattern (0xA302 EXIF SubIFD, 0x828E TIFF/EP) as Undefined type
        0xA302 | 0x828E if data.len() >= 4 => {
            if let Some(formatted) = format_cfa_pattern(data) {
                Value::String(formatted)
            } else {
                Value::String(format_binary_data(data))
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
                Value::String(format_binary_data(data))
            }
        }
    }
}

/// Special handling for RetouchHistory (Nikon 0x009E) - return "None" if all zeros
#[cfg(feature = "serde")]
fn format_retouch_history(values: &[u16]) -> Option<Value> {
    if values.iter().all(|&v| v == 0) {
        Some(Value::String("None".to_string()))
    } else {
        // Return decoded values for non-zero entries
        let decoded: Vec<&str> = values
            .iter()
            .filter(|&&v| v != 0)
            .map(|&v| crate::makernotes::nikon::decode_retouch_history_exiftool(v))
            .collect();
        if decoded.is_empty() {
            Some(Value::String("None".to_string()))
        } else {
            Some(Value::String(decoded.join(", ")))
        }
    }
}

/// Format GPS coordinate (latitude or longitude) as DMS string
/// Converts rational array [(41,1), (21,1), (4832,100)] to "41 deg 21' 48.32\" N"
#[cfg(feature = "serde")]
fn format_gps_coordinate(coords: &[(u32, u32)], ref_value: Option<&str>) -> String {
    if coords.len() < 3 {
        return String::new();
    }

    let deg = if coords[0].1 != 0 {
        coords[0].0 as f64 / coords[0].1 as f64
    } else {
        0.0
    };
    let min = if coords[1].1 != 0 {
        coords[1].0 as f64 / coords[1].1 as f64
    } else {
        0.0
    };
    let sec = if coords[2].1 != 0 {
        coords[2].0 as f64 / coords[2].1 as f64
    } else {
        0.0
    };

    // Format with appropriate precision for seconds
    let sec_str = if sec.fract() == 0.0 {
        format!("{:.0}", sec)
    } else {
        // Remove trailing zeros but keep at least 2 decimal places
        let formatted = format!("{:.2}", sec);
        formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    };

    if let Some(ref_val) = ref_value {
        format!(
            "{} deg {}' {}\" {}",
            deg as i32,
            min as i32,
            sec_str,
            ref_val.trim()
        )
    } else {
        format!("{} deg {}' {}\"", deg as i32, min as i32, sec_str)
    }
}

/// Format GPS timestamp as HH:MM:SS
/// Converts rational array [(9,1), (53,1), (44,1)] to "09:53:44"
#[cfg(feature = "serde")]
fn format_gps_timestamp(time: &[(u32, u32)]) -> String {
    if time.len() < 3 {
        return String::new();
    }

    let h = if time[0].1 != 0 {
        time[0].0 / time[0].1
    } else {
        0
    };
    let m = if time[1].1 != 0 {
        time[1].0 / time[1].1
    } else {
        0
    };
    let s = if time[2].1 != 0 {
        time[2].0 / time[2].1
    } else {
        0
    };

    format!("{:02}:{:02}:{:02}", h, m, s)
}

/// Format CFA pattern bytes as ExifTool-compatible string
/// Converts [0,1,1,2] to "[Red,Green][Green,Blue]"
/// EXIF CFAPattern (0xA302) format: 2 bytes horiz repeat, 2 bytes vert repeat, then pattern
/// TIFF/EP CFAPattern (0x828E) format: just the 4 pattern bytes
#[cfg(feature = "serde")]
fn format_cfa_pattern(data: &[u8]) -> Option<String> {
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

    // EXIF CFAPattern (0xA302) has 4-byte dimension prefix + pattern
    // Format: [horiz_repeat:u16][vert_repeat:u16][pattern bytes]
    // For 2x2 pattern: 8 bytes total (e.g., 0x00 0x02 0x00 0x02 + 4 pattern bytes)
    // Endianness varies by camera - try both BE and LE
    if data.len() == 8 {
        // Try big-endian first (0x00 0x02 0x00 0x02)
        let horiz_be = u16::from_be_bytes([data[0], data[1]]);
        let vert_be = u16::from_be_bytes([data[2], data[3]]);
        if horiz_be == 2 && vert_be == 2 {
            // Pattern is in bytes 4-7
            return Some(format!(
                "[{},{}][{},{}]",
                color_name(data[4]),
                color_name(data[5]),
                color_name(data[6]),
                color_name(data[7])
            ));
        }
        // Try little-endian (0x02 0x00 0x02 0x00)
        let horiz_le = u16::from_le_bytes([data[0], data[1]]);
        let vert_le = u16::from_le_bytes([data[2], data[3]]);
        if horiz_le == 2 && vert_le == 2 {
            // Pattern is in bytes 4-7
            return Some(format!(
                "[{},{}][{},{}]",
                color_name(data[4]),
                color_name(data[5]),
                color_name(data[6]),
                color_name(data[7])
            ));
        }
    }

    // TIFF/EP CFAPattern (0x828E) or other 4-byte patterns
    if data.len() >= 4 {
        return Some(format!(
            "[{},{}][{},{}]",
            color_name(data[0]),
            color_name(data[1]),
            color_name(data[2]),
            color_name(data[3])
        ));
    }

    None
}

/// Check if a string is a pure numeric value that can be output as a JSON number
/// Returns true for integers (including negatives) and decimals
#[cfg(feature = "serde")]
fn is_numeric_string(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // Handle optional leading minus sign
    let s = s.strip_prefix('-').unwrap_or(s);

    if s.is_empty() {
        return false;
    }

    // Check for decimal point
    if let Some(dot_pos) = s.find('.') {
        // Check integer part (before dot)
        let int_part = &s[..dot_pos];
        let frac_part = &s[dot_pos + 1..];

        // Integer part must have at least one digit (or be empty for ".5" style)
        if !int_part.is_empty() {
            if !int_part.chars().all(|c| c.is_ascii_digit()) {
                return false;
            }
            // No leading zeros except for "0.x"
            if int_part.len() > 1 && int_part.starts_with('0') {
                return false;
            }
        }

        // Fraction part must have at least one digit
        if frac_part.is_empty() || !frac_part.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }

        return true;
    }

    // Pure integer - check if all characters are digits
    if !s.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    // Check for leading zeros (but "0" itself is fine)
    if s.len() > 1 && s.starts_with('0') {
        return false;
    }
    true
}

/// Convert an ExifValue to a JSON value in exiftool-compatible format
#[cfg(feature = "serde")]
pub fn format_exif_value_for_json(value: &ExifValue, tag_id: u16) -> Value {
    format_exif_value_for_json_with_make(value, tag_id, None)
}

/// Convert an ExifValue to a JSON value in exiftool-compatible format with manufacturer info
#[cfg(feature = "serde")]
pub fn format_exif_value_for_json_with_make(
    value: &ExifValue,
    tag_id: u16,
    make: Option<&str>,
) -> Value {
    match value {
        // ASCII strings - return as number if purely numeric, otherwise string
        ExifValue::Ascii(s) => {
            let cleaned = s.trim_end_matches('\0').trim();
            // ExifTool outputs pure numeric strings (like SerialNumber, SubSecTime)
            // as JSON numbers, so we do the same
            if is_numeric_string(cleaned) {
                // Try parsing as unsigned integer first
                if let Ok(n) = cleaned.parse::<u64>() {
                    return Value::Number(n.into());
                }
                // Try parsing as signed integer
                if let Ok(n) = cleaned.parse::<i64>() {
                    return Value::Number(n.into());
                }
                // Try parsing as float
                if let Ok(f) = cleaned.parse::<f64>() {
                    if let Some(num) = serde_json::Number::from_f64(f) {
                        return Value::Number(num);
                    }
                }
            }
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

        // Single-value Short - use helper function with make for manufacturer-specific decoding
        ExifValue::Short(v) if v.len() == 1 => format_short_value_with_make(v[0], tag_id, make),

        // Single-value other numeric types
        ExifValue::Long(v) if v.len() == 1 => Value::Number(v[0].into()),
        ExifValue::SByte(v) if v.len() == 1 => Value::Number(v[0].into()),
        ExifValue::SShort(v) if v.len() == 1 => Value::Number(v[0].into()),
        ExifValue::SLong(v) if v.len() == 1 => Value::Number(v[0].into()),

        // Single-value Float/Double
        ExifValue::Float(v) if v.len() == 1 => serde_json::Number::from_f64(v[0] as f64)
            .map(Value::Number)
            .unwrap_or_else(|| Value::String(v[0].to_string())),
        ExifValue::Double(v) if v.len() == 1 => {
            let val = v[0];
            // Output as integer if value is a whole number (like MinAperture: 36 not 36.0)
            if val.fract() == 0.0 && val.abs() < i64::MAX as f64 {
                Value::Number((val as i64).into())
            } else {
                serde_json::Number::from_f64(val)
                    .map(Value::Number)
                    .unwrap_or_else(|| Value::String(val.to_string()))
            }
        }

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
            } else {
                // Default to space-separated strings (ExifTool behavior)
                Value::String(
                    v.iter()
                        .map(|n| n.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                )
            }
        }
        ExifValue::Short(v) => {
            // Nikon RetouchHistory (0x009E) - special handling
            if tag_id == 0x009E {
                if let Some(formatted) = format_retouch_history(v) {
                    return formatted;
                }
            }
            // Fuji WhiteBalanceFineTune (0x100A) - format as "Red +X, Blue +Y"
            if tag_id == 0x100A && v.len() == 2 {
                let red = v[0] as i16;
                let blue = v[1] as i16;
                let red_str = if red >= 0 {
                    format!("+{}", red)
                } else {
                    format!("{}", red)
                };
                let blue_str = if blue >= 0 {
                    format!("+{}", blue)
                } else {
                    format!("{}", blue)
                };
                return Value::String(format!("Red {}, Blue {}", red_str, blue_str));
            }
            // Default to space-separated strings (ExifTool behavior)
            Value::String(
                v.iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        }
        ExifValue::Long(v) => {
            // Fuji ImageStabilization (0x1422) - format as "Type; Mode; Param"
            if tag_id == 0x1422 && v.len() >= 3 {
                let is_type = match v[0] {
                    0 => "None",
                    1 => "Optical",
                    2 => "Sensor-shift",
                    3 => "OIS/IBIS Combined",
                    _ => "Unknown",
                };
                let is_mode = match v[1] {
                    0 => "Off",
                    1 => "On (mode 1, continuous)",
                    2 => "On (mode 2, shooting only)",
                    3 => "On (mode 3, panning)",
                    _ => "Unknown",
                };
                return Value::String(format!("{}; {}; {}", is_type, is_mode, v[2]));
            }
            // Default to space-separated strings (ExifTool behavior)
            Value::String(
                v.iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        }
        ExifValue::SByte(v) => Value::String(
            v.iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        ),
        ExifValue::SShort(v) => {
            // Fuji WhiteBalanceFineTune (0x100A) - format as "Red +X, Blue +Y"
            if tag_id == 0x100A && v.len() == 2 {
                let red = v[0];
                let blue = v[1];
                let red_str = if red >= 0 {
                    format!("+{}", red)
                } else {
                    format!("{}", red)
                };
                let blue_str = if blue >= 0 {
                    format!("+{}", blue)
                } else {
                    format!("{}", blue)
                };
                return Value::String(format!("Red {}, Blue {}", red_str, blue_str));
            }
            // Default to space-separated strings (ExifTool behavior)
            Value::String(
                v.iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        }
        ExifValue::SLong(v) => Value::String(
            v.iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        ),

        // Multi-value Float/Double - space-separated strings (ExifTool behavior)
        ExifValue::Float(v) => Value::String(
            v.iter()
                .map(|&f| f.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        ),
        ExifValue::Double(v) => Value::String(
            v.iter()
                .map(|&f| f.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        ),

        // Multi-value Rationals - format as space-separated decimals (ExifTool behavior)
        ExifValue::Rational(v) => {
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
        }
        ExifValue::SRational(v) => {
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

    // Extract Make for manufacturer-specific formatting (tag 0x010F)
    let make: Option<String> = exif_data.get_tag_by_id(0x010F).and_then(|v| match v {
        ExifValue::Ascii(s) => Some(s.trim_end_matches('\0').trim().to_string()),
        _ => None,
    });
    let make_ref = make.as_deref();

    // Convert each tag to a key-value pair
    for (tag_id, value) in exif_data.iter() {
        let tag_name = if let Some(name) = tag_id.name() {
            name.to_string()
        } else {
            format!("Tag{}", tag_id.id)
        };
        let json_value = format_exif_value_for_json_with_make(value, tag_id.id, make_ref);
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

    // Format GPS coordinates with DMS (degrees, minutes, seconds) and direction
    // GPSLatitude (0x0002) with GPSLatitudeRef (0x0001)
    if let Some(ExifValue::Rational(coords)) = exif_data.get_tag_by_id(0x0002) {
        if coords.len() >= 3 {
            let ref_value = exif_data.get_tag_by_id(0x0001).and_then(|v| match v {
                ExifValue::Ascii(s) => Some(s.as_str()),
                _ => None,
            });
            let formatted = format_gps_coordinate(coords, ref_value);
            output.insert("GPSLatitude".to_string(), Value::String(formatted));
        }
    }

    // GPSLongitude (0x0004) with GPSLongitudeRef (0x0003)
    if let Some(ExifValue::Rational(coords)) = exif_data.get_tag_by_id(0x0004) {
        if coords.len() >= 3 {
            let ref_value = exif_data.get_tag_by_id(0x0003).and_then(|v| match v {
                ExifValue::Ascii(s) => Some(s.as_str()),
                _ => None,
            });
            let formatted = format_gps_coordinate(coords, ref_value);
            output.insert("GPSLongitude".to_string(), Value::String(formatted));
        }
    }

    // GPSTimeStamp (0x0007) - format as HH:MM:SS
    if let Some(ExifValue::Rational(time)) = exif_data.get_tag_by_id(0x0007) {
        if time.len() >= 3 {
            let formatted = format_gps_timestamp(time);
            output.insert("GPSTimeStamp".to_string(), Value::String(formatted));
        }
    }

    // GPSAltitude (0x0006) with GPSAltitudeRef (0x0005) - format as "226.6 m Above Sea Level"
    if let Some(ExifValue::Rational(alt)) = exif_data.get_tag_by_id(0x0006) {
        if !alt.is_empty() && alt[0].1 != 0 {
            let altitude = alt[0].0 as f64 / alt[0].1 as f64;
            let ref_desc = exif_data.get_tag_by_id(0x0005).and_then(|v| match v {
                ExifValue::Byte(b) if !b.is_empty() => {
                    Some(crate::tags::get_gps_altitude_ref_description(b[0]))
                }
                _ => None,
            });
            let formatted = if let Some(ref_str) = ref_desc {
                format!("{:.1} m {}", altitude, ref_str)
            } else {
                format!("{:.1} m", altitude)
            };
            output.insert("GPSAltitude".to_string(), Value::String(formatted));
        }
    }

    // GPSLatitudeRef (0x0001) - expand N/S to North/South
    if let Some(ExifValue::Ascii(ref_val)) = exif_data.get_tag_by_id(0x0001) {
        let expanded = crate::tags::get_gps_latitude_ref_description(ref_val.trim());
        output.insert(
            "GPSLatitudeRef".to_string(),
            Value::String(expanded.to_string()),
        );
    }

    // GPSLongitudeRef (0x0003) - expand E/W to East/West
    if let Some(ExifValue::Ascii(ref_val)) = exif_data.get_tag_by_id(0x0003) {
        let expanded = crate::tags::get_gps_longitude_ref_description(ref_val.trim());
        output.insert(
            "GPSLongitudeRef".to_string(),
            Value::String(expanded.to_string()),
        );
    }

    // Add derived date fields for ExifTool compatibility
    // ModifyDate is an alias for DateTime (0x0132)
    if let Some(ExifValue::Ascii(s)) = exif_data.get_tag_by_id(0x0132) {
        let cleaned = s.trim_end_matches('\0').trim();
        output.insert("ModifyDate".to_string(), Value::String(cleaned.to_string()));
    }

    // CreateDate is an alias for DateTimeDigitized (0x9004)
    if let Some(ExifValue::Ascii(s)) = exif_data.get_tag_by_id(0x9004) {
        let cleaned = s.trim_end_matches('\0').trim();
        output.insert("CreateDate".to_string(), Value::String(cleaned.to_string()));
    }

    // Add maker notes if present
    // MakerNote tags can override EXIF tags for certain fields where the MakerNote
    // value is more accurate (like MeteringMode for Canon cameras)
    if let Some(maker_notes) = exif_data.get_maker_notes() {
        // Tags where MakerNote should override EXIF (ExifTool behavior)
        // Note: Saturation/Contrast/Sharpness removed - use EXIF standard values for all
        const MAKERNOTE_PRIORITY_TAGS: &[&str] = &["MeteringMode", "WhiteBalance", "LightSource"];

        for (tag_id, maker_tag) in maker_notes.iter() {
            let tag_name = maker_tag
                .tag_name
                .unwrap_or_else(|| Box::leak(format!("MakerNote{:04X}", tag_id).into_boxed_str()));

            // Allow MakerNote to override EXIF for priority tags
            let should_insert = if MAKERNOTE_PRIORITY_TAGS.contains(&tag_name) {
                true // Always use MakerNote value
            } else {
                !output.contains_key(tag_name) // Only add if not already present
            };

            if should_insert {
                let json_value = format_exif_value_for_json(&maker_tag.value, *tag_id);
                output.insert(tag_name.to_string(), json_value);
            }
        }
    }

    // Add computed fields for exiftool compatibility

    // LensID computation:
    // For Nikon: Use LensModel if available (contains full lens name like "1 NIKKOR 10mm f/2.8")
    // For Canon: Use LensType as LensID (the lens type lookup value)
    if !output.contains_key("LensID") {
        let is_canon = make_ref
            .map(|m| m.to_uppercase().contains("CANON"))
            .unwrap_or(false);

        if is_canon {
            // Canon: LensID should come from LensType (the decoded lens type value)
            if let Some(lens_type) = output.get("LensType").cloned() {
                output.insert("LensID".to_string(), lens_type);
            }
        } else {
            // Nikon and others: Use LensModel if available
            if let Some(lens_model) = output.get("LensModel").cloned() {
                output.insert("LensID".to_string(), lens_model);
            } else if let Some(lens_type) = output.get("LensType").cloned() {
                output.insert("LensID".to_string(), lens_type);
            }
        }
    }

    // Lens - compute from MinFocalLength and MaxFocalLength
    if !output.contains_key("Lens") {
        let min_fl = output.get("MinFocalLength").and_then(|v| {
            if let Value::String(s) = v {
                s.trim_end_matches(" mm").parse::<f64>().ok()
            } else {
                None
            }
        });
        let max_fl = output.get("MaxFocalLength").and_then(|v| {
            if let Value::String(s) = v {
                s.trim_end_matches(" mm").parse::<f64>().ok()
            } else {
                None
            }
        });
        if let (Some(min), Some(max)) = (min_fl, max_fl) {
            // If min and max are the same (prime lens), just show one value
            let lens = if (min - max).abs() < 0.01 {
                format!("{:.1} mm", min)
            } else {
                format!("{:.1} - {:.1} mm", min, max)
            };
            output.insert("Lens".to_string(), Value::String(lens));
        }
    }

    // Add ExifImageWidth/Height aliases for PixelXDimension/PixelYDimension
    // ExifTool uses "ExifImageWidth" for tag 0xA002
    if let Some(pxd) = output.get("PixelXDimension").cloned() {
        output.insert("ExifImageWidth".to_string(), pxd);
    }
    if let Some(pyd) = output.get("PixelYDimension").cloned() {
        output.insert("ExifImageHeight".to_string(), pyd);
    }

    // Add RAF-specific metadata if present (for Fujifilm RAF files)
    // This must come before ImageSize calculation so we can use RawImageCroppedWidth/Height
    if let Some(raf_metadata) = exif_data.get_raf_metadata() {
        for (key, value) in &raf_metadata.tags {
            // Try to parse as number for fields that should be numeric
            if key == "RawImageWidth"
                || key == "RawImageHeight"
                || key == "RawImageFullWidth"
                || key == "RawImageFullHeight"
                || key == "RawImageCroppedWidth"
                || key == "RawImageCroppedHeight"
            {
                if let Ok(n) = value.parse::<i64>() {
                    output.insert(key.clone(), Value::Number(n.into()));
                } else {
                    output.insert(key.clone(), Value::String(value.clone()));
                }
            } else {
                output.insert(key.clone(), Value::String(value.clone()));
            }
        }
    }

    // ImageSize - compute from actual image dimensions
    // Priority order:
    // 1. RawImageCroppedWidth/Height - RAF metadata for Fuji (actual cropped image size)
    // 2. PanasonicImageWidth/Height - MakerNote values for actual image dimensions
    // 3. PixelXDimension/PixelYDimension (EXIF tags 0xA002/0xA003) - may be thumbnail in RAW
    // 4. ImageWidth/ImageLength - TIFF tags, often thumbnail dimensions in raw files
    let width = output
        .get("RawImageCroppedWidth")
        .or_else(|| output.get("PanasonicImageWidth"))
        .or_else(|| output.get("PixelXDimension"))
        .or_else(|| output.get("ImageWidth"))
        .and_then(|v| match v {
            Value::Number(n) => n.as_u64(),
            _ => None,
        });
    let height = output
        .get("RawImageCroppedHeight")
        .or_else(|| output.get("PanasonicImageHeight"))
        .or_else(|| output.get("PixelYDimension"))
        .or_else(|| output.get("ImageHeight"))
        .or_else(|| output.get("ImageLength"))
        .and_then(|v| match v {
            Value::Number(n) => n.as_u64(),
            _ => None,
        });
    if let (Some(w), Some(h)) = (width, height) {
        if !output.contains_key("ImageSize") {
            output.insert(
                "ImageSize".to_string(),
                Value::String(format!("{}x{}", w, h)),
            );
        }
        if !output.contains_key("Megapixels") {
            let mp = (w * h) as f64 / 1_000_000.0;
            let rounded = (mp * 10.0).round() / 10.0;
            if let Some(num) = serde_json::Number::from_f64(rounded) {
                output.insert("Megapixels".to_string(), Value::Number(num));
            }
        }
    }

    // Add ExifByteOrder - indicates TIFF byte order
    let byte_order_str = match exif_data.endian {
        crate::data_types::Endianness::Little => "Little-endian (Intel, II)",
        crate::data_types::Endianness::Big => "Big-endian (Motorola, MM)",
    };
    output.insert(
        "ExifByteOrder".to_string(),
        Value::String(byte_order_str.to_string()),
    );

    // Compute RedBalance and BlueBalance from WB_GRBLevels if available
    // WB_GRBLevels format is "G R B" (green, red, blue values)
    // RedBalance = R / G, BlueBalance = B / G
    if let Some(Value::String(wb_levels)) = output.get("WB_GRBLevels") {
        let parts: Vec<f64> = wb_levels
            .split_whitespace()
            .filter_map(|s| s.parse::<f64>().ok())
            .collect();
        if parts.len() >= 3 && parts[0] != 0.0 {
            let g = parts[0];
            let r = parts[1];
            let b = parts[2];
            if !output.contains_key("RedBalance") {
                let red_balance = r / g;
                if let Some(num) = serde_json::Number::from_f64(red_balance) {
                    output.insert("RedBalance".to_string(), Value::Number(num));
                }
            }
            if !output.contains_key("BlueBalance") {
                let blue_balance = b / g;
                if let Some(num) = serde_json::Number::from_f64(blue_balance) {
                    output.insert("BlueBalance".to_string(), Value::Number(num));
                }
            }
        }
    }

    // Wrap in an array like exiftool does
    Value::Array(vec![Value::Object(output)])
}
