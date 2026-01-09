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

/// Round to specified decimal places using banker's rounding (round half to even)
/// This matches Perl's default rounding behavior used by ExifTool
fn round_half_even(x: f64, decimals: i32) -> f64 {
    let multiplier = 10f64.powi(decimals);
    let shifted = x * multiplier;
    let floor = shifted.floor();
    let frac = shifted - floor;
    if (frac - 0.5).abs() < 1e-9 {
        // Exactly 0.5 - round to even
        if floor as i64 % 2 == 0 {
            floor / multiplier
        } else {
            (floor + 1.0) / multiplier
        }
    } else {
        shifted.round() / multiplier
    }
}

/// Format a float with up to 10 significant digits (ExifTool precision)
/// Removes trailing zeros after decimal point
fn format_float_10_sig(val: f64) -> String {
    if val == 0.0 {
        return "0".to_string();
    }
    // Special case for integers
    if val.fract() == 0.0 && val.abs() < 1e10 {
        return format!("{:.0}", val);
    }
    // Format with 10 significant digits using {:.*e} then convert back
    // This ensures we get exactly 10 significant digits regardless of magnitude
    let abs_val = val.abs();
    let log10 = abs_val.log10().floor() as i32;
    // Number of decimal places needed for 10 sig digits
    let decimals = (9 - log10).max(0) as usize;
    let formatted = format!("{:.*}", decimals, val);
    // Trim trailing zeros and unnecessary decimal point
    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
    trimmed.to_string()
}

/// Format EXIF MeteringMode - use manufacturer-specific naming where appropriate
#[cfg(feature = "serde")]
fn format_metering_mode(value: u16, make: Option<&str>) -> String {
    if let Some(m) = make {
        let make_upper = m.to_uppercase();
        // Canon uses "Evaluative" instead of "Multi-segment", "Default" instead of "Unknown"
        if make_upper.contains("CANON") {
            return match value {
                0 => "Default",
                1 => "Average",
                2 => "Center-weighted average",
                3 => "Spot",
                4 => "Multi-spot",
                5 => "Evaluative",
                6 => "Partial",
                255 => "Other",
                _ => "Unknown",
            }
            .to_string();
        }
        // Note: Olympus uses "ESP" in MakerNotes but ExifTool uses standard "Multi-segment"
        // for the EXIF MeteringMode tag (0x9207), so we don't apply special handling here.
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
        // DNG CFALayout (0xC617)
        0xC617 => Value::String(crate::tags::get_cfa_layout_description(value).to_string()),
        // FocalLengthIn35mmFormat - add "mm" suffix
        0xA405 => Value::String(format!("{} mm", value)),
        // Sony ARW SubIFD tags
        0x7030 => Value::String(
            crate::tags::get_sony_vignetting_correction_description(value).to_string(),
        ),
        0x7034 => Value::String(
            crate::tags::get_sony_chromatic_aberration_correction_description(value).to_string(),
        ),
        0x7036 => Value::String(
            crate::tags::get_sony_distortion_correction_description(value).to_string(),
        ),
        _ => Value::Number(value.into()),
    }
}

/// Format a single Rational value with tag-specific interpretation
#[cfg(feature = "serde")]
fn format_rational_value(num: u32, den: u32, tag_id: u16) -> Value {
    if den == 0 {
        // Match ExifTool: 0/0 = "undef", n/0 (n!=0) = "inf"
        return if num == 0 {
            Value::String("undef".to_string())
        } else {
            Value::String("inf".to_string())
        };
    }

    match tag_id {
        0x829A => {
            // ExposureTime - ExifTool outputs:
            // - Integer for whole seconds (25 -> 25)
            // - Decimal for non-standard times (0.625 -> 0.6, 4.8 -> 4.8)
            // - Fraction 1/n for standard shutter speeds
            let exposure_time = num as f64 / den as f64;
            if exposure_time >= 1.0 {
                // For exposures >= 1 second
                if (exposure_time - exposure_time.round()).abs() < 0.01 {
                    // Whole second - output as integer
                    Value::Number((exposure_time.round() as u32).into())
                } else {
                    // Fractional seconds - output as decimal with 1 decimal place
                    let rounded = (exposure_time * 10.0).round() / 10.0;
                    serde_json::Number::from_f64(rounded)
                        .map(Value::Number)
                        .unwrap_or_else(|| Value::Number((exposure_time.round() as u32).into()))
                }
            } else if exposure_time > 0.0 {
                // For sub-second exposures
                let approx_den = (1.0 / exposure_time).round() as u32;
                // Check if 1/n is a good approximation (within 10% to match ExifTool's rounding)
                let approx_time = 1.0 / approx_den as f64;
                if approx_den > 1 && (approx_time - exposure_time).abs() / exposure_time < 0.10 {
                    Value::String(format!("1/{}", approx_den))
                } else {
                    // Use decimal for non-standard times
                    let rounded = (exposure_time * 10.0).round() / 10.0;
                    serde_json::Number::from_f64(rounded)
                        .map(Value::Number)
                        .unwrap_or_else(|| Value::Number(0.into()))
                }
            } else {
                Value::Number(0.into())
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
        // Match ExifTool: 0/0 = "undef", n/0 (n!=0) = "inf"
        return if num == 0 {
            Value::String("undef".to_string())
        } else {
            Value::String("inf".to_string())
        };
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
                    // Whole number - ExifTool outputs negative as number (-1),
                    // but positive as string with + prefix ("+1")
                    let int_val = decimal.round() as i64;
                    if int_val < 0 {
                        Value::Number(int_val.into())
                    } else {
                        Value::String(format!("+{}", int_val))
                    }
                } else {
                    // Decimal format (e.g., +0.33, -0.67)
                    // ExifTool outputs negative decimals as numbers, positive as strings with + prefix
                    let rounded = (decimal * 100.0).round() / 100.0;
                    if rounded < 0.0 {
                        // Negative decimal - output as number
                        serde_json::Number::from_f64(rounded)
                            .map(Value::Number)
                            .unwrap_or_else(|| {
                                Value::String(
                                    format!("{:.2}", rounded)
                                        .trim_end_matches('0')
                                        .trim_end_matches('.')
                                        .to_string(),
                                )
                            })
                    } else {
                        // Positive decimal - output as string with + prefix
                        let formatted = format!("+{:.2}", rounded)
                            .trim_end_matches('0')
                            .trim_end_matches('.')
                            .to_string();
                        Value::String(formatted)
                    }
                }
            }
        }
        // Nikon ExposureBracketValue (0x0019) - output as fraction using PrintFraction algorithm
        // ExifTool outputs: "0", "+N", "-N", "+N/2", "-N/2", "+N/3", "-N/3", or decimal with + prefix
        0x0019 => {
            let decimal = num as f64 / den as f64;
            // Avoid round-off errors like ExifTool does (val *= 1.00001)
            let val = decimal * 1.00001;
            if val.abs() < 0.0001 {
                Value::Number(0.into())
            } else if (val.trunc() / val).abs() > 0.999 {
                // Whole number: +N or -N
                Value::String(format!("{:+}", val.trunc() as i32))
            } else if ((val * 2.0).trunc() / (val * 2.0)).abs() > 0.999 {
                // Half: +N/2 or -N/2
                Value::String(format!("{:+}/2", (val * 2.0).trunc() as i32))
            } else if ((val * 3.0).trunc() / (val * 3.0)).abs() > 0.999 {
                // Third: +N/3 or -N/3
                Value::String(format!("{:+}/3", (val * 3.0).trunc() as i32))
            } else {
                // Fallback to decimal with + prefix for positive
                Value::String(format!("{:+.3}", decimal))
            }
        }
        // Nikon MakerNote exposure-related tags (packed rational decode)
        // ProgramShift (0x000D), ExposureDifference (0x000E), FlashExposureComp (0x0012),
        // ExternalFlashExposureComp (0x0017), FlashExposureBracketValue (0x0018), ExposureTuning (0x001C)
        // ExifTool outputs: 0 as int (except FlashExposureBracketValue which is 0.0),
        // negative as number (int for whole numbers), positive as string with "+" (fraction for 1/3 steps)
        0x000D | 0x000E | 0x0012 | 0x0017 | 0x0018 | 0x001C => {
            let decimal = num as f64 / den as f64;
            // Round to 1 decimal place using banker's rounding to match ExifTool
            let rounded = round_half_even(decimal, 1);
            if rounded == 0.0 {
                // FlashExposureBracketValue (0x0018) outputs 0.0, others output 0
                if tag_id == 0x0018 {
                    serde_json::Number::from_f64(0.0)
                        .map(Value::Number)
                        .unwrap_or_else(|| Value::Number(0.into()))
                } else {
                    Value::Number(0.into())
                }
            } else if rounded > 0.0 {
                // Positive: output as string with "+" prefix
                // Use fraction format for common EV steps (1/3, 2/3)
                let frac = rounded.fract().abs();
                let whole = rounded.trunc() as i32;
                let formatted = if (frac - 0.3).abs() < 0.05 {
                    if whole == 0 {
                        "+1/3".to_string()
                    } else {
                        format!("+{} 1/3", whole)
                    }
                } else if (frac - 0.7).abs() < 0.05 {
                    if whole == 0 {
                        "+2/3".to_string()
                    } else {
                        format!("+{} 2/3", whole)
                    }
                } else {
                    format!("+{:.1}", rounded)
                        .trim_end_matches('0')
                        .trim_end_matches('.')
                        .to_string()
                };
                Value::String(formatted)
            } else {
                // Negative: output as number
                // ExposureDifference (0x000E) always uses float, others use int for whole numbers
                if rounded.fract() == 0.0 && tag_id != 0x000E {
                    Value::Number((rounded as i64).into())
                } else {
                    serde_json::Number::from_f64(rounded)
                        .map(Value::Number)
                        .unwrap_or_else(|| Value::Number(0.into()))
                }
            }
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
        // CFAPattern (0xA302 EXIF SubIFD) as Undefined type - format as [Red,Green][Green,Blue]
        0xA302 if data.len() >= 4 => {
            if let Some(formatted) = format_cfa_pattern(data) {
                Value::String(formatted)
            } else {
                Value::String(format_binary_data(data))
            }
        }
        // CFAPattern2 (0x828E TIFF/EP) - output as raw space-separated values
        0x828E => Value::String(
            data.iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        ),
        // TIFF-EPStandardID (0x9216) - format as version string "1.0.0.0"
        0x9216 if data.len() == 4 => {
            Value::String(format!("{}.{}.{}.{}", data[0], data[1], data[2], data[3]))
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
    // ExifTool always uses 2 decimal places for fractional seconds
    let sec_str = if sec.fract() == 0.0 {
        format!("{:.0}", sec)
    } else {
        format!("{:.2}", sec)
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
    format_exif_value_for_json_with_make_and_name(value, tag_id, None, None)
}

/// Convert an ExifValue to a JSON value in exiftool-compatible format with tag name
/// Tag name allows for special handling of certain tags like InternalSerialNumber
#[cfg(feature = "serde")]
pub fn format_exif_value_for_json_with_name(
    value: &ExifValue,
    tag_id: u16,
    tag_name: Option<&str>,
) -> Value {
    format_exif_value_for_json_with_make_and_name(value, tag_id, None, tag_name)
}

/// Convert an ExifValue to a JSON value in exiftool-compatible format with manufacturer info
#[cfg(feature = "serde")]
pub fn format_exif_value_for_json_with_make(
    value: &ExifValue,
    tag_id: u16,
    make: Option<&str>,
) -> Value {
    format_exif_value_for_json_with_make_and_name(value, tag_id, make, None)
}

/// Convert an ExifValue to a JSON value in exiftool-compatible format with manufacturer and tag name info
#[cfg(feature = "serde")]
pub fn format_exif_value_for_json_with_make_and_name(
    value: &ExifValue,
    tag_id: u16,
    make: Option<&str>,
    tag_name: Option<&str>,
) -> Value {
    match value {
        // ASCII strings - return as number if purely numeric, otherwise string
        ExifValue::Ascii(s) => {
            // For Olympus InternalSerialNumber, keep trailing spaces (ExifTool pads to 32 chars)
            let is_olympus_internal_serial = tag_name == Some("InternalSerialNumber")
                && make
                    .map(|m| m.to_uppercase().contains("OLYMPUS"))
                    .unwrap_or(false);

            let cleaned = if is_olympus_internal_serial {
                // Only trim null bytes, keep trailing spaces
                s.trim_end_matches('\0')
            } else {
                s.trim_end_matches('\0').trim()
            };
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
        ExifValue::SShort(v) if v.len() == 1 => {
            // Sony ARW SubIFD correction tags are stored as SShort
            match tag_id {
                0x7030 => Value::String(
                    crate::tags::get_sony_vignetting_correction_description(v[0] as u16)
                        .to_string(),
                ),
                0x7034 => Value::String(
                    crate::tags::get_sony_chromatic_aberration_correction_description(v[0] as u16)
                        .to_string(),
                ),
                0x7036 => Value::String(
                    crate::tags::get_sony_distortion_correction_description(v[0] as u16)
                        .to_string(),
                ),
                _ => Value::Number(v[0].into()),
            }
        }
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
            // GPSVersionID (0x0000), DNGVersion (0xC612), DNGBackwardVersion (0xC613),
            // TIFF-EPStandardID (0x9216) should be formatted as "1.0.0.0"
            if (tag_id == 0x0000 || tag_id == 0xC612 || tag_id == 0xC613 || tag_id == 0x9216)
                && v.len() == 4
            {
                Value::String(format!("{}.{}.{}.{}", v[0], v[1], v[2], v[3]))
            // CFAPattern (0xA302 EXIF) - format as [Red,Green][Green,Blue]
            } else if tag_id == 0xA302 && v.len() >= 4 {
                if let Some(formatted) = format_cfa_pattern(v) {
                    Value::String(formatted)
                } else {
                    Value::Array(v.iter().map(|&n| Value::Number(n.into())).collect())
                }
            // CFAPattern2 (0x828E TIFF/EP) - output as raw space-separated values
            } else if tag_id == 0x828E {
                Value::String(
                    v.iter()
                        .map(|n| n.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                )
            // CFAPlaneColor (0xC616 DNG) - convert indices to color names
            } else if tag_id == 0xC616 {
                Value::String(crate::tags::get_cfa_plane_color_description(v))
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
            // YCbCrSubSampling (0x0212) - format as "YCbCr4:X:X (h v)"
            if tag_id == 0x0212 && v.len() == 2 {
                let key = format!("{} {}", v[0], v[1]);
                let desc = match key.as_str() {
                    "1 1" => "YCbCr4:4:4 (1 1)",
                    "2 1" => "YCbCr4:2:2 (2 1)",
                    "2 2" => "YCbCr4:2:0 (2 2)",
                    "4 1" => "YCbCr4:1:1 (4 1)",
                    "4 2" => "YCbCr4:1:0 (4 2)",
                    "1 2" => "YCbCr4:4:0 (1 2)",
                    "1 4" => "YCbCr4:4:1 (1 4)",
                    "2 4" => "YCbCr4:2:1 (2 4)",
                    _ => return Value::String(key),
                };
                return Value::String(desc.to_string());
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
            // StripOffsets (0x0111) and StripByteCounts (0x0117) - format as binary data for large arrays
            // ExifTool outputs "(Binary data N bytes, use -b option to extract)" for these
            // N is the text representation size (decimal strings + spaces), not binary size
            if (tag_id == 0x0111 || tag_id == 0x0117) && v.len() > 1 {
                // Calculate text representation size: sum of decimal lengths + spaces
                let text_len: usize = v.iter().map(|n| n.to_string().len()).sum::<usize>()
                    + v.len().saturating_sub(1); // spaces between values
                return Value::String(format!(
                    "(Binary data {} bytes, use -b option to extract)",
                    text_len
                ));
            }
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
        // Use 10 significant digits to match ExifTool precision
        ExifValue::Rational(v) => {
            let decimals: Vec<String> = v
                .iter()
                .map(|&(num, den)| {
                    if den == 0 {
                        // Match ExifTool: 0/0 = "undef", n/0 (n!=0) = "inf"
                        if num == 0 {
                            "undef".to_string()
                        } else {
                            "inf".to_string()
                        }
                    } else {
                        let decimal = num as f64 / den as f64;
                        format_float_10_sig(decimal)
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
                        // Match ExifTool: 0/0 = "undef", n/0 (n!=0) = "inf"
                        if num == 0 {
                            "undef".to_string()
                        } else {
                            "inf".to_string()
                        }
                    } else {
                        let decimal = num as f64 / den as f64;
                        format_float_10_sig(decimal)
                    }
                })
                .collect();
            Value::String(decimals.join(" "))
        }

        // Undefined - use helper function
        ExifValue::Undefined(v) => format_undefined_value(v, tag_id),
    }
}

/// Get image dimensions from EXIF data, prioritizing IFD0 tags
/// Returns (width, height) as Options
/// Priority: IFD0 ImageWidth/Length (0x0100/0x0101) > ExifImageWidth/Height (0xa002/0xa003)
#[cfg(feature = "serde")]
fn get_image_dimensions(exif_data: &ExifData) -> (Option<u64>, Option<u64>) {
    // Helper to extract dimension value
    fn extract_dimension(value: Option<&ExifValue>) -> Option<u64> {
        match value {
            Some(ExifValue::Long(v)) if !v.is_empty() => Some(v[0] as u64),
            Some(ExifValue::Short(v)) if !v.is_empty() => Some(v[0] as u64),
            _ => None,
        }
    }

    // Try IFD0 ImageWidth (0x0100) and ImageLength (0x0101) first
    // These are the actual raw image dimensions for most cameras
    let ifd0_width = extract_dimension(exif_data.get_tag_by_id(0x0100));
    let ifd0_height = extract_dimension(exif_data.get_tag_by_id(0x0101));

    if ifd0_width.is_some() && ifd0_height.is_some() {
        return (ifd0_width, ifd0_height);
    }

    // Fall back to ExifImageWidth/Height (0xa002/0xa003)
    let exif_width = extract_dimension(exif_data.get_tag_by_id(0xa002));
    let exif_height = extract_dimension(exif_data.get_tag_by_id(0xa003));

    (exif_width.or(ifd0_width), exif_height.or(ifd0_height))
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
    // Process Main IFD first to give it priority, then other IFDs only if tag not already present
    // This matches ExifTool behavior where IFD0 (Main) tags take precedence
    for (tag_id, value) in exif_data.iter() {
        if tag_id.ifd != crate::tags::TagGroup::Main {
            continue;
        }
        let tag_name = if let Some(name) = tag_id.name() {
            name.to_string()
        } else {
            format!("Tag{}", tag_id.id)
        };
        if tag_id.id == 0xA302 {
            continue;
        }
        let json_value = format_exif_value_for_json_with_make(value, tag_id.id, make_ref);
        output.insert(tag_name, json_value);
    }
    // Then process other IFDs (Exif, GPS, Interop), skipping Thumbnail for most tags
    for (tag_id, value) in exif_data.iter() {
        if tag_id.ifd == crate::tags::TagGroup::Main {
            continue; // Already processed
        }
        // Skip Thumbnail IFD for tags that should come from Main IFD
        // ExifTool prioritizes IFD0 for Compression, BitsPerSample, etc.
        if tag_id.ifd == crate::tags::TagGroup::Thumbnail {
            continue;
        }
        let tag_name = if let Some(name) = tag_id.name() {
            name.to_string()
        } else {
            format!("Tag{}", tag_id.id)
        };
        // Skip 0xA302 (EXIF CFAPattern) - we'll generate CFAPattern composite from CFAPattern2 (0x828E)
        // ExifTool generates CFAPattern from CFAPattern2, not from 0xA302
        if tag_id.id == 0xA302 {
            continue;
        }
        // Only add if not already present from Main IFD
        if !output.contains_key(&tag_name) {
            let json_value = format_exif_value_for_json_with_make(value, tag_id.id, make_ref);
            output.insert(tag_name, json_value);
        }
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

    // CFAPattern composite - generate from CFAPattern2 (0x828E) or fallback to 0xA302
    // ExifTool generates CFAPattern from CFAPattern2 when available
    if !output.contains_key("CFAPattern") {
        // Try CFAPattern2 (0x828E) first - 4 bytes of pattern values
        let pattern_values: Option<Vec<u8>> =
            exif_data.get_tag_by_id(0x828E).and_then(|v| match v {
                ExifValue::Byte(b) if b.len() >= 4 => Some(b.clone()),
                ExifValue::Short(s) if s.len() >= 4 => Some(s.iter().map(|&x| x as u8).collect()),
                ExifValue::Undefined(u) if u.len() >= 4 => Some(u.clone()),
                _ => None,
            });

        // Fallback to EXIF CFAPattern (0xA302) - 8 bytes: 4 for dimensions + 4 for pattern
        let pattern_values = pattern_values.or_else(|| {
            exif_data.get_tag_by_id(0xA302).and_then(|v| match v {
                ExifValue::Undefined(u) if u.len() >= 8 => Some(u[4..8].to_vec()),
                _ => None,
            })
        });

        if let Some(pattern) = pattern_values {
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
            let formatted = format!(
                "[{},{}][{},{}]",
                color_name(pattern[0]),
                color_name(pattern[1]),
                color_name(pattern[2]),
                color_name(pattern[3])
            );
            output.insert("CFAPattern".to_string(), Value::String(formatted));
        }
    }

    // ImageSize and Megapixels - computed from image dimensions
    // For Sony: Use IFD0 ImageWidth/Length (0x0100/0x0101) which matches ExifTool
    // For other brands: Let the later code handle MakerNote-specific dimensions
    let is_sony = make_ref
        .map(|m| m.to_uppercase().contains("SONY"))
        .unwrap_or(false);
    if is_sony {
        let (width, height) = get_image_dimensions(exif_data);
        if let (Some(w), Some(h)) = (width, height) {
            output.insert(
                "ImageSize".to_string(),
                Value::String(format!("{}x{}", w, h)),
            );
            let megapixels = (w as f64 * h as f64) / 1_000_000.0;
            // Round to 1 decimal place
            let rounded = (megapixels * 10.0).round() / 10.0;
            if let Some(num) = serde_json::Number::from_f64(rounded) {
                output.insert("Megapixels".to_string(), Value::Number(num));
            }
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
            // ExifTool uses integer for whole numbers, 1 decimal otherwise
            let alt_str = if altitude.fract() == 0.0 {
                format!("{:.0}", altitude)
            } else {
                format!("{:.1}", altitude)
            };
            let formatted = if let Some(ref_str) = ref_desc {
                format!("{} m {}", alt_str, ref_str)
            } else {
                format!("{} m", alt_str)
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

    // GPSStatus (0x0009) - expand A/V to Measurement Active/Void
    if let Some(ExifValue::Ascii(status)) = exif_data.get_tag_by_id(0x0009) {
        let expanded = match status.trim() {
            "A" => "Measurement Active",
            "V" => "Measurement Void",
            other => other,
        };
        output.insert("GPSStatus".to_string(), Value::String(expanded.to_string()));
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

    // SubSec composite date tags - date + subsec
    // SubSecModifyDate = ModifyDate + SubSecTime
    let subsec_time = exif_data.get_tag_by_id(0x9290).and_then(|v| match v {
        ExifValue::Ascii(s) => Some(s.trim_end_matches('\0').trim().to_string()),
        _ => None,
    });
    let subsec_original = exif_data.get_tag_by_id(0x9291).and_then(|v| match v {
        ExifValue::Ascii(s) => Some(s.trim_end_matches('\0').trim().to_string()),
        _ => None,
    });
    let subsec_digitized = exif_data.get_tag_by_id(0x9292).and_then(|v| match v {
        ExifValue::Ascii(s) => Some(s.trim_end_matches('\0').trim().to_string()),
        _ => None,
    });

    // SubSecModifyDate = ModifyDate (0x0132) + SubSecTime (0x9290)
    if let (Some(ExifValue::Ascii(date)), Some(subsec)) =
        (exif_data.get_tag_by_id(0x0132), &subsec_time)
    {
        let cleaned = date.trim_end_matches('\0').trim();
        output.insert(
            "SubSecModifyDate".to_string(),
            Value::String(format!("{}.{}", cleaned, subsec)),
        );
    }

    // SubSecDateTimeOriginal = DateTimeOriginal (0x9003) + SubSecTimeOriginal (0x9291)
    if let (Some(ExifValue::Ascii(date)), Some(subsec)) =
        (exif_data.get_tag_by_id(0x9003), &subsec_original)
    {
        let cleaned = date.trim_end_matches('\0').trim();
        output.insert(
            "SubSecDateTimeOriginal".to_string(),
            Value::String(format!("{}.{}", cleaned, subsec)),
        );
    }

    // SubSecCreateDate = DateTimeDigitized (0x9004) + SubSecTimeDigitized (0x9292)
    if let (Some(ExifValue::Ascii(date)), Some(subsec)) =
        (exif_data.get_tag_by_id(0x9004), &subsec_digitized)
    {
        let cleaned = date.trim_end_matches('\0').trim();
        output.insert(
            "SubSecCreateDate".to_string(),
            Value::String(format!("{}.{}", cleaned, subsec)),
        );
    }

    // Add maker notes if present
    // MakerNote tags can override EXIF tags for certain fields where the MakerNote
    // value is more accurate (like MeteringMode for Canon cameras)
    if let Some(maker_notes) = exif_data.get_maker_notes() {
        // Tags where MakerNote should override EXIF (ExifTool behavior)
        // FocusMode added: Sony has multiple FocusMode tags (0x201B, 0xB042, 0xB04E) where
        // higher IDs are used for newer cameras and should take precedence
        // Note: Saturation/Contrast/Sharpness NOT included - only CameraSettings sub-tags
        // (for A200/A300/A350/A700/A850/A900) should override EXIF
        const MAKERNOTE_PRIORITY_TAGS: &[&str] =
            &["MeteringMode", "WhiteBalance", "LightSource", "FocusMode"];

        // Sort by tag_id to ensure consistent ordering (lower IDs processed first)
        // This is important for Olympus where CameraSettings ImageStabilization (0x2624)
        // should take priority over FocusInfo ImageStabilization (0x3650)
        let mut sorted_tags: Vec<_> = maker_notes.iter().collect();
        sorted_tags.sort_by_key(|(tag_id, _)| *tag_id);

        for (tag_id, maker_tag) in sorted_tags {
            let tag_name = maker_tag
                .tag_name
                .unwrap_or_else(|| Box::leak(format!("MakerNote{:04X}", tag_id).into_boxed_str()));

            // Allow MakerNote to override EXIF for priority tags
            // Also allow Sony CameraSettings sub-tags (0xC01C-0xC01E) to override EXIF
            // These are extracted from the CameraSettings blob for A200/A300/A350/A700/A850/A900
            let is_sony_camera_settings_subtag = *tag_id >= 0xC01C
                && *tag_id <= 0xC01E
                && make_ref.is_some_and(|m| m.contains("SONY"));
            let should_insert =
                if MAKERNOTE_PRIORITY_TAGS.contains(&tag_name) || is_sony_camera_settings_subtag {
                    true // Always use MakerNote value
                } else {
                    !output.contains_key(tag_name) // Only add if not already present
                };

            if should_insert {
                let json_value = format_exif_value_for_json_with_make_and_name(
                    &maker_tag.value,
                    *tag_id,
                    make_ref,
                    Some(tag_name),
                );
                output.insert(tag_name.to_string(), json_value);
            }
        }
    }

    // Add computed fields for exiftool compatibility

    // LensID computation:
    // For Nikon: Use LensModel if available (contains full lens name like "1 NIKKOR 10mm f/2.8")
    // For Canon/Olympus: Use LensType as LensID (the lens type lookup value)
    // For Sony: Use LensType2 for E-mount lenses, LensType for A-mount
    let is_canon = make_ref
        .map(|m| m.to_uppercase().contains("CANON"))
        .unwrap_or(false);
    let is_olympus = make_ref
        .map(|m| m.to_uppercase().contains("OLYMPUS"))
        .unwrap_or(false);
    let is_sony = make_ref
        .map(|m| m.to_uppercase().contains("SONY"))
        .unwrap_or(false);

    // For Sony, check if existing LensID is the generic E-mount placeholder
    // If so, replace it with LensType2 if available
    if is_sony {
        let should_replace = if let Some(Value::String(lens_id)) = output.get("LensID") {
            lens_id.contains("E-Mount")
                || lens_id.contains("T-Mount")
                || lens_id.contains("Other Lens")
        } else {
            !output.contains_key("LensID")
        };

        if should_replace {
            if let Some(lens_type2) = output.get("LensType2").cloned() {
                output.insert("LensID".to_string(), lens_type2);
            } else if let Some(lens_type) = output.get("LensType").cloned() {
                // Only use LensType if it's not the generic E-mount placeholder
                if let Value::String(s) = &lens_type {
                    if !s.contains("E-Mount") && !s.contains("T-Mount") && !s.contains("Other Lens")
                    {
                        output.insert("LensID".to_string(), lens_type);
                    }
                } else {
                    output.insert("LensID".to_string(), lens_type);
                }
            }
        }
    } else if !output.contains_key("LensID") {
        if is_canon || is_olympus {
            // Canon/Olympus: LensID should come from LensType (the decoded lens type value)
            if let Some(lens_type) = output.get("LensType").cloned() {
                output.insert("LensID".to_string(), lens_type);
            }
        } else {
            // Nikon and others: Use LensModel if available
            if let Some(lens_model) = output.get("LensModel").cloned() {
                // Fujifilm: Add space before F followed by digit (e.g., "XF55-200mmF3.5" -> "XF55-200mm F3.5")
                let is_fuji = make_ref
                    .map(|m| m.to_uppercase().contains("FUJI"))
                    .unwrap_or(false);
                let lens_id = if is_fuji {
                    if let Value::String(s) = &lens_model {
                        // Insert space before F followed by digit if not already present
                        let fixed = s.replace("mmF", "mm F").replace("MMF", "MM F");
                        Value::String(fixed)
                    } else {
                        lens_model
                    }
                } else {
                    lens_model
                };
                output.insert("LensID".to_string(), lens_id);
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

    // LensSpec composite - generated from Lens + LensType (for Nikon cameras)
    // Format: "17-55mm f/2.8 G" from Lens="17-55mm f/2.8" and LensType="G"
    if !output.contains_key("LensSpec") {
        let lens = output.get("Lens").and_then(|v| v.as_str());
        let lens_type = output.get("LensType").and_then(|v| v.as_str());

        if let (Some(l), Some(t)) = (lens, lens_type) {
            // Only add LensType if it's meaningful (not empty)
            let lens_spec = if t.is_empty() {
                l.to_string()
            } else {
                format!("{} {}", l, t)
            };
            output.insert("LensSpec".to_string(), Value::String(lens_spec));
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
            } else if key == "RawExposureBias" {
                // RawExposureBias should be a number in JSON (e.g., -0.7)
                if let Ok(n) = value.parse::<f64>() {
                    if let Some(num) = serde_json::Number::from_f64(n) {
                        output.insert(key.clone(), Value::Number(num));
                    } else {
                        output.insert(key.clone(), Value::String(value.clone()));
                    }
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

    // Also try WB_RGGBLevels format (R G G B) - common in Canon
    if !output.contains_key("RedBalance") || !output.contains_key("BlueBalance") {
        if let Some(Value::String(wb_levels)) = output.get("WB_RGGBLevels") {
            let parts: Vec<f64> = wb_levels
                .split_whitespace()
                .filter_map(|s| s.parse::<f64>().ok())
                .collect();
            if parts.len() >= 4 {
                let r = parts[0];
                let g1 = parts[1];
                let g2 = parts[2];
                let b = parts[3];
                let g = (g1 + g2) / 2.0;
                if g != 0.0 {
                    if !output.contains_key("RedBalance") {
                        if let Some(num) = serde_json::Number::from_f64(r / g) {
                            output.insert("RedBalance".to_string(), Value::Number(num));
                        }
                    }
                    if !output.contains_key("BlueBalance") {
                        if let Some(num) = serde_json::Number::from_f64(b / g) {
                            output.insert("BlueBalance".to_string(), Value::Number(num));
                        }
                    }
                }
            }
        }
    }

    // ImageHeight - alias for ImageLength (TIFF uses ImageLength, ExifTool uses ImageHeight)
    if !output.contains_key("ImageHeight") {
        if let Some(val) = output.get("ImageLength").cloned() {
            output.insert("ImageHeight".to_string(), val);
        }
    }

    // InteropIndex - format as "R98 - DCF basic file (sRGB)" or "THM - DCF thumbnail file"
    if let Some(Value::String(idx)) = output.get("InteropIndex").cloned() {
        let formatted = match idx.as_str() {
            "R98" => "R98 - DCF basic file (sRGB)",
            "THM" => "THM - DCF thumbnail file",
            "R03" => "R03 - DCF option file (Adobe RGB)",
            _ => &idx,
        };
        output.insert(
            "InteropIndex".to_string(),
            Value::String(formatted.to_string()),
        );
    }

    // InteropVersion - convert "30 31 30 30" hex bytes to "0100"
    if let Some(Value::String(ver)) = output.get("InteropVersion").cloned() {
        if ver.contains(' ') {
            // It's hex bytes like "30 31 30 30" - convert to ASCII string
            let decoded: String = ver
                .split_whitespace()
                .filter_map(|hex| u8::from_str_radix(hex, 16).ok())
                .filter(|&b| b.is_ascii_alphanumeric())
                .map(|b| b as char)
                .collect();
            if !decoded.is_empty() {
                output.insert("InteropVersion".to_string(), Value::String(decoded));
            }
        }
    }

    // ============================================================================
    // Calculated/Composite fields (like ExifTool's Composite tags)
    // ============================================================================

    // Helper function to extract f64 from JSON value
    fn parse_f64_value(v: &Value) -> Option<f64> {
        match v {
            Value::Number(n) => n.as_f64(),
            Value::String(s) => {
                let s = s
                    .trim()
                    .trim_end_matches(" mm")
                    .trim_end_matches(" s")
                    .trim_end_matches(" m");
                // Try parsing as fraction (e.g., "1/800")
                if let Some(slash_pos) = s.find('/') {
                    let num = s[..slash_pos].parse::<f64>().ok()?;
                    let denom = s[slash_pos + 1..].parse::<f64>().ok()?;
                    if denom != 0.0 {
                        return Some(num / denom);
                    }
                }
                // Try parsing as plain number
                s.parse::<f64>().ok()
            }
            _ => None,
        }
    }

    // Extract all values we need first (before any inserts)
    let focal_length = output.get("FocalLength").and_then(parse_f64_value);
    // For computed fields (HyperfocalDistance, DOF, LightValue), use FNumber first
    // to match ExifTool behavior. Aperture is a derived value that may differ slightly.
    let aperture = output
        .get("FNumber")
        .and_then(parse_f64_value)
        .or_else(|| output.get("Aperture").and_then(parse_f64_value));
    let exposure_time = output.get("ExposureTime").and_then(parse_f64_value);
    let iso = output
        .get("ISO")
        .and_then(parse_f64_value)
        .or_else(|| output.get("ISOSpeedRatings").and_then(parse_f64_value));
    let fl35_raw = output
        .get("FocalLengthIn35mmFilm")
        .or_else(|| output.get("FocalLengthIn35mmFormat"))
        .and_then(parse_f64_value);
    let min_fl = output.get("MinFocalLength").and_then(parse_f64_value);
    let max_fl = output.get("MaxFocalLength").and_then(parse_f64_value);
    // Get focus distance - matching ExifTool's priority:
    // 1. FocusDistance if available
    // 2. SubjectDistance, ObjectDistance, or ApproximateFocusDistance
    // 3. Average of FocusDistanceLower and FocusDistanceUpper
    let focus_dist_str: Option<Value> = output
        .get("FocusDistance")
        .or_else(|| output.get("SubjectDistance"))
        .or_else(|| output.get("ObjectDistance"))
        .or_else(|| output.get("ApproximateFocusDistance"))
        .cloned()
        .or_else(|| {
            // If we have both Lower and Upper, compute the average
            let lower = output.get("FocusDistanceLower").and_then(|v| {
                if let Value::String(s) = v {
                    s.trim_end_matches(" m").parse::<f64>().ok()
                } else {
                    None
                }
            });
            let upper = output.get("FocusDistanceUpper").and_then(|v| {
                if let Value::String(s) = v {
                    s.trim_end_matches(" m").parse::<f64>().ok()
                } else {
                    None
                }
            });
            match (lower, upper) {
                (Some(l), Some(u)) => {
                    let avg = (l + u) / 2.0;
                    Some(Value::String(format!("{:.2} m", avg)))
                }
                (Some(l), None) => Some(Value::String(format!("{:.2} m", l))),
                (None, Some(u)) => Some(Value::String(format!("{:.2} m", u))),
                (None, None) => None,
            }
        });

    // For ScaleFactor calculation from sensor size
    let image_width = output.get("ExifImageWidth").and_then(parse_f64_value);
    let image_height = output.get("ExifImageHeight").and_then(parse_f64_value);
    let focal_plane_x_res = output
        .get("FocalPlaneXResolution")
        .and_then(parse_f64_value);
    let focal_plane_y_res = output
        .get("FocalPlaneYResolution")
        .and_then(parse_f64_value);
    // FocalPlaneResolutionUnit: can be numeric (2=inches, 3=cm) or string ("inches", "cm")
    let focal_plane_res_unit = output
        .get("FocalPlaneResolutionUnit")
        .and_then(|v| match v {
            Value::Number(n) => n.as_f64(),
            Value::String(s) => match s.to_lowercase().as_str() {
                "inches" => Some(2.0),
                "cm" => Some(3.0),
                "mm" => Some(4.0),
                _ => s.parse::<f64>().ok(),
            },
            _ => None,
        });
    // MakerNote sensor size tags
    let sensor_width = output.get("SensorWidth").and_then(parse_f64_value);
    let sensor_height = output.get("SensorHeight").and_then(parse_f64_value);
    let focal_plane_diag = output.get("FocalPlaneDiagonal").and_then(parse_f64_value);
    // FocalPlaneXSize/YSize from Canon CRW/CR2 files (sensor dimensions in mm)
    let focal_plane_x_size = output.get("FocalPlaneXSize").and_then(parse_f64_value);
    let focal_plane_y_size = output.get("FocalPlaneYSize").and_then(parse_f64_value);
    // Nikon SensorPixelSize: "7.8 7.8" = width height in micrometers
    let sensor_pixel_size = output.get("SensorPixelSize").and_then(|v| {
        if let Value::String(s) = v {
            let parts: Vec<f64> = s
                .split_whitespace()
                .filter_map(|p| p.parse().ok())
                .collect();
            if parts.len() >= 2 {
                Some((parts[0], parts[1]))
            } else {
                None
            }
        } else {
            None
        }
    });

    let has_shutter_speed = output.contains_key("ShutterSpeed");
    let has_scale_factor = output.contains_key("ScaleFactor35efl");
    let has_focal_length_35 = output.contains_key("FocalLength35efl");
    let has_lens_35 = output.contains_key("Lens35efl");
    let has_coc = output.contains_key("CircleOfConfusion");
    let has_hyperfocal = output.contains_key("HyperfocalDistance");
    let has_fov = output.contains_key("FOV");
    let has_lv = output.contains_key("LightValue");
    let has_dof = output.contains_key("DOF");

    // Now do all inserts

    // ShutterSpeed - from ExposureTime or ShutterSpeedValue
    if !has_shutter_speed {
        if let Some(exp_time) = exposure_time {
            if exp_time > 0.0 {
                let formatted = if exp_time >= 1.0 {
                    format!("{}", exp_time.round() as u32)
                } else {
                    format!("1/{}", (1.0 / exp_time).round() as u32)
                };
                output.insert("ShutterSpeed".to_string(), Value::String(formatted));
            }
        }
    }

    // ScaleFactor35efl - crop factor relative to 35mm full frame
    // Full frame diagonal = sqrt(36² + 24²) = 43.2666mm
    const FF_DIAG: f64 = 43.2666;

    // Canon-specific sensor diagonal calculation
    // Canon stores sensor dimensions in FocalPlaneResolution denominators:
    // - numerator = image_pixels × 1000
    // - denominator = sensor_dimension_inches × 1000
    // So sensor_diagonal_mm = sqrt(denom_x² + denom_y²) × 0.0254
    let canon_sensor_diag: Option<f64> = if make_ref
        .map(|m| m.to_uppercase().contains("CANON"))
        .unwrap_or(false)
    {
        // Get raw FocalPlaneResolution rational values (tag 0xA20E, 0xA20F)
        let x_rational = exif_data.get_tag_by_id(0xA20E).and_then(|v| match v {
            ExifValue::Rational(r) if !r.is_empty() => Some(r[0]),
            _ => None,
        });
        let y_rational = exif_data.get_tag_by_id(0xA20F).and_then(|v| match v {
            ExifValue::Rational(r) if !r.is_empty() => Some(r[0]),
            _ => None,
        });

        if let (Some((x_num, x_den)), Some((y_num, y_den))) = (x_rational, y_rational) {
            // Validate Canon's format:
            // - numerators divisible by 1000
            // - image >= 640x480 (numerator >= 640000)
            // - sensor between 0.061" and 1.5" (denominator 61-1500)
            // - sensor not square (denominators different)
            if x_num % 1000 == 0
                && y_num % 1000 == 0
                && (640000..10000000).contains(&x_num)
                && (480000..10000000).contains(&y_num)
                && (61..1500).contains(&x_den)
                && (61..1000).contains(&y_den)
                && x_den != y_den
            {
                // Denominators are sensor size in inches × 1000
                let diag_mm = ((x_den * x_den + y_den * y_den) as f64).sqrt() * 0.0254;
                Some(diag_mm)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Try multiple sources in order of preference:
    // 1. FocalLengthIn35mmFilm / FocalLength (most accurate)
    // 2. Canon sensor diagonal from FocalPlaneResolution denominators
    // 3. FocalPlaneDiagonal from MakerNotes (Olympus, etc.)
    // 4. SensorWidth/Height from MakerNotes (Canon, Panasonic, etc.)
    // 5. SensorPixelSize from MakerNotes (Nikon) + image dimensions
    // 6. FocalPlaneResolution + image dimensions (fallback)
    let scale_factor = if let (Some(fl35), Some(fl)) = (fl35_raw, focal_length) {
        if fl > 0.0 && fl35 > 0.0 {
            Some(fl35 / fl)
        } else {
            None
        }
    } else if let Some(diag) = canon_sensor_diag {
        // Canon-specific calculation using FocalPlaneResolution denominators
        Some(FF_DIAG / diag)
    } else if let Some(diag) = focal_plane_diag {
        // FocalPlaneDiagonal is sensor diagonal in mm
        if diag > 0.0 {
            Some(FF_DIAG / diag)
        } else {
            None
        }
    } else if let (Some(xsize), Some(ysize)) = (focal_plane_x_size, focal_plane_y_size) {
        // FocalPlaneXSize/YSize are sensor dimensions in mm (Canon CRW/CR2)
        let diag = (xsize * xsize + ysize * ysize).sqrt();
        if diag > 0.0 {
            Some(FF_DIAG / diag)
        } else {
            None
        }
    } else if let (Some(sw), Some(sh)) = (sensor_width, sensor_height) {
        // SensorWidth/Height are in pixels - need to convert to mm
        // For Canon, these are actual sensor dimensions in some unit
        // Check if values look like mm (typically 15-40mm for APS-C/FF)
        let diag = (sw * sw + sh * sh).sqrt();
        if diag > 10.0 && diag < 50.0 {
            // Values are likely in mm
            Some(FF_DIAG / diag)
        } else if let (Some(_w), Some(_h), Some(xres), Some(yres)) = (
            image_width,
            image_height,
            focal_plane_x_res,
            focal_plane_y_res,
        ) {
            // SensorWidth/Height are in pixels, use focal plane res to convert
            let unit_to_mm = match focal_plane_res_unit.unwrap_or(2.0) as u32 {
                2 => 25.4,
                3 => 10.0,
                4 => 1.0,
                5 => 0.001,
                _ => 25.4,
            };
            // Scale sensor dimensions by pixel ratio
            let sw_mm = sw * unit_to_mm / xres;
            let sh_mm = sh * unit_to_mm / yres;
            let sensor_diag = (sw_mm * sw_mm + sh_mm * sh_mm).sqrt();
            if sensor_diag > 0.0 {
                Some(FF_DIAG / sensor_diag)
            } else {
                None
            }
        } else {
            None
        }
    } else if let (Some((px_w, px_h)), Some(w), Some(h)) =
        (sensor_pixel_size, image_width, image_height)
    {
        // Nikon SensorPixelSize: calculate sensor size from pixel size (µm) and image dimensions
        // sensor_mm = pixels × pixel_size_µm / 1000
        let sw_mm = w * px_w / 1000.0;
        let sh_mm = h * px_h / 1000.0;
        let sensor_diag = (sw_mm * sw_mm + sh_mm * sh_mm).sqrt();
        if sensor_diag > 0.0 {
            Some(FF_DIAG / sensor_diag)
        } else {
            None
        }
    } else if let (Some(w), Some(h), Some(xres), Some(yres)) = (
        image_width,
        image_height,
        focal_plane_x_res,
        focal_plane_y_res,
    ) {
        // Calculate sensor dimensions from focal plane resolution
        let unit_to_mm = match focal_plane_res_unit.unwrap_or(2.0) as u32 {
            2 => 25.4,
            3 => 10.0,
            4 => 1.0,
            5 => 0.001,
            _ => 25.4,
        };
        let sw_mm = w * unit_to_mm / xres;
        let sh_mm = h * unit_to_mm / yres;
        let sensor_diag = (sw_mm * sw_mm + sh_mm * sh_mm).sqrt();
        if sensor_diag > 0.0 {
            Some(FF_DIAG / sensor_diag)
        } else {
            None
        }
    } else {
        None
    };

    if let Some(sf) = scale_factor {
        if !has_scale_factor && sf > 0.5 && sf < 10.0 {
            if let Some(num) = serde_json::Number::from_f64((sf * 10.0).round() / 10.0) {
                output.insert("ScaleFactor35efl".to_string(), Value::Number(num));
            }
        }
    }

    // FocalLength35efl - focal length with 35mm equivalent
    // ExifTool format: "XX.X mm (35 mm equivalent: YY.Y mm)" when scale factor known
    // Otherwise just: "XX.X mm"
    if !has_focal_length_35 {
        if let Some(fl) = focal_length {
            // We have original focal length - check if we have scale factor too
            if let Some(sf) = scale_factor {
                // Calculate 35mm equivalent
                let fl35 = fl * sf;
                // Full format with both original and equivalent
                let formatted = format!("{:.1} mm (35 mm equivalent: {:.1} mm)", fl, fl35);
                output.insert("FocalLength35efl".to_string(), Value::String(formatted));
            } else {
                // No scale factor - just output the focal length
                let formatted = format!("{:.1} mm", fl);
                output.insert("FocalLength35efl".to_string(), Value::String(formatted));
            }
        } else if let Some(fl35) = fl35_raw {
            // Only have 35mm equivalent from EXIF tag - output as-is
            let formatted = format!("{:.1} mm", fl35);
            output.insert("FocalLength35efl".to_string(), Value::String(formatted));
        }
    }

    // Lens35efl - lens focal range with 35mm equivalent
    // Format: "min - max mm (35 mm equivalent: min35 - max35 mm)"
    if !has_lens_35 {
        if let Some(sf) = scale_factor {
            if let (Some(min), Some(max)) = (min_fl, max_fl) {
                let min35 = min * sf;
                let max35 = max * sf;
                let formatted = if (min - max).abs() < 0.01 {
                    // Single focal length lens (prime)
                    format!("{:.1} mm (35 mm equivalent: {:.1} mm)", min, min35)
                } else {
                    // Zoom lens
                    format!(
                        "{:.1} - {:.1} mm (35 mm equivalent: {:.1} - {:.1} mm)",
                        min, max, min35, max35
                    )
                };
                output.insert("Lens35efl".to_string(), Value::String(formatted));
            }
        }
    }

    // CircleOfConfusion - sensor diagonal / 1440
    // Full frame diagonal = sqrt(24² + 36²) = 43.27mm
    // CoC = 43.27 / (ScaleFactor × 1440)
    let coc = scale_factor.map(|sf| 43.27 / (sf * 1440.0));
    if let Some(c) = coc {
        if !has_coc {
            let formatted = format!("{:.3} mm", c);
            output.insert("CircleOfConfusion".to_string(), Value::String(formatted));
        }
    }

    // HyperfocalDistance = FocalLength² / (Aperture × CoC × 1000)
    // Matches ExifTool's formula (omits the +FocalLength term for simplicity)
    if !has_hyperfocal {
        if let (Some(fl), Some(ap), Some(c)) = (focal_length, aperture, coc) {
            if ap > 0.0 && c > 0.0 {
                // ExifTool formula: f² / (N × c × 1000) where f and c are in mm, result in meters
                let hyper_m = (fl * fl) / (ap * c * 1000.0);
                let formatted = format!("{:.2} m", hyper_m);
                output.insert("HyperfocalDistance".to_string(), Value::String(formatted));
            }
        }
    }

    // FOV (Field of View) - matches ExifTool's calculation
    // Uses focus distance correction factor when available
    // Format: "XX.X deg" or "XX.X deg (YY.YY m)" with distance
    if !has_fov {
        if let (Some(fl), Some(sf)) = (focal_length, scale_factor) {
            if fl > 0.0 {
                // Parse focus distance in meters
                let focus_m: Option<f64> = focus_dist_str.as_ref().and_then(|v| {
                    if let Value::String(s) = v {
                        s.trim_end_matches(" m").parse().ok()
                    } else {
                        None
                    }
                });

                // Calculate correction factor based on focus distance (ExifTool algorithm)
                let corr = if let Some(fd) = focus_m {
                    let d = 1000.0 * fd - fl; // convert distance to mm and subtract focal length
                    if d > 0.0 {
                        1.0 + fl / d
                    } else {
                        1.0
                    }
                } else {
                    1.0
                };

                // ExifTool: atan2(36, 2*FocalLength*ScaleFactor*corr) * 360 / PI
                // This is half FOV angle, multiplied by 360/PI to get full FOV in degrees
                let half_fov = (36.0 / (2.0 * fl * sf * corr)).atan();
                let fov_deg = half_fov * 360.0 / std::f64::consts::PI;

                // Calculate FOV width at focus distance if available and reasonable
                let formatted = if let Some(fd) = focus_m {
                    if fd > 0.0 && fd < 10000.0 {
                        // FOV width = 2 * FocusDistance * tan(half_fov)
                        let fov_width = 2.0 * fd * half_fov.tan();
                        format!("{:.1} deg ({:.2} m)", fov_deg, fov_width)
                    } else {
                        format!("{:.1} deg", fov_deg)
                    }
                } else {
                    format!("{:.1} deg", fov_deg)
                };
                output.insert("FOV".to_string(), Value::String(formatted));
            }
        }
    }

    // LightValue = 2 × log2(Aperture) - log2(ShutterSpeed) - log2(ISO/100)
    if !has_lv {
        if let (Some(ap), Some(exp), Some(iso_val)) = (aperture, exposure_time, iso) {
            if ap > 0.0 && exp > 0.0 && iso_val > 0.0 {
                let lv = 2.0 * ap.log2() - exp.log2() - (iso_val / 100.0).log2();
                if let Some(num) = serde_json::Number::from_f64((lv * 10.0).round() / 10.0) {
                    output.insert("LightValue".to_string(), Value::Number(num));
                }
            }
        }
    }

    // DOF (Depth of Field) - requires focus distance
    // Format: "DOF m (near - far m)" or "inf (near m - inf)"
    if !has_dof {
        if let Some(Value::String(s)) = focus_dist_str {
            if let Ok(focus_m) = s.trim_end_matches(" m").parse::<f64>() {
                if let (Some(fl), Some(ap), Some(c)) = (focal_length, aperture, coc) {
                    let focus_mm = focus_m * 1000.0;
                    if ap > 0.0 && focus_mm > 0.0 && focus_mm.is_finite() {
                        // Edge case: focus < focal_length (physically impossible but matches ExifTool)
                        // In this case, near and far both equal focus distance, DOF = 0
                        if focus_mm <= fl {
                            let formatted = format!("-0.00 m ({:.2} - {:.2} m)", focus_m, focus_m);
                            output.insert("DOF".to_string(), Value::String(formatted));
                        } else {
                            let hyper = (fl * fl) / (ap * c) + fl;
                            let near = (hyper * focus_mm) / (hyper + (focus_mm - fl));
                            let far = if focus_mm < hyper {
                                (hyper * focus_mm) / (hyper - (focus_mm - fl))
                            } else {
                                f64::INFINITY
                            };
                            let dof = far - near;
                            let near_m = near / 1000.0;
                            let far_m = far / 1000.0;
                            let formatted = if dof.is_infinite() {
                                format!("inf ({:.2} m - inf)", near_m)
                            } else {
                                format!("{:.2} m ({:.2} - {:.2} m)", dof / 1000.0, near_m, far_m)
                            };
                            output.insert("DOF".to_string(), Value::String(formatted));
                        }
                    }
                }
            }
        }
    }

    // Clean up output to match ExifTool behavior:
    // 1. Remove duplicate tags where we have both EXIF name and ExifTool alias
    // 2. Remove IFD pointer tags (ExifTool doesn't output these)
    // 3. Remove raw binary data tags (ExifTool processes these instead of outputting raw)

    // Tags to remove (EXIF names that have ExifTool aliases or are duplicates)
    let duplicate_tags = [
        "DateTime",          // -> ModifyDate
        "DateTimeDigitized", // -> CreateDate
        "ISOSpeedRatings",   // -> ISO (already handled by duplicate insertion)
        "ImageLength",       // -> ImageHeight (already handled)
        "PixelXDimension",   // -> ExifImageWidth (duplicate)
        "PixelYDimension",   // -> ExifImageHeight (duplicate)
    ];

    // IFD pointer tags and structure tags (not output by ExifTool)
    let pointer_tags = [
        "ExifOffset",
        "InteroperabilityIFDPointer",
        "GPSInfo",
        "FocusInfoIFD",
        "ImageProcessingIFD",
        "RawDevelopmentIFD",
        "RawInfoIFD",
        "SubIFDs",
        "JPEGInterchangeFormat",       // IFD0/IFD1 thumbnail offset
        "JPEGInterchangeFormatLength", // IFD0/IFD1 thumbnail length
        "NewSubfileType",              // IFD subfile type
    ];

    // Raw binary tags (ExifTool processes these instead of outputting raw)
    let binary_tags = [
        "MakerNote",
        "ThumbnailPrintIM",
        "XMPMetadata",
        "PrintIM",
        // Canon-specific raw binary tags
        "CanonAFInfo",
        "CanonCustomFunctions",
        "CanonFlashInfo",
        "CRWParam",
        "MeasuredColor",
        "PersonalFunctionValues",
        // Olympus binary tags (ExifTool marks as Binary with variable byte counts)
        "FaceDetectArea",
    ];

    // Remove all the static tags
    for tag in duplicate_tags
        .iter()
        .chain(pointer_tags.iter())
        .chain(binary_tags.iter())
    {
        output.remove(*tag);
    }

    // Remove unnamed MakerNote tags (e.g., MakerNote0206, MakerNote100F)
    // These are internal tags that ExifTool doesn't output
    let unnamed_makernote_tags: Vec<String> = output
        .keys()
        .filter(|k| {
            k.starts_with("MakerNote")
                && k.len() > 9
                && k[9..].chars().all(|c| c.is_ascii_hexdigit())
        })
        .cloned()
        .collect();
    for tag in unnamed_makernote_tags {
        output.remove(&tag);
    }

    // Remove GPS tags with invalid/placeholder values
    // ExifTool doesn't output GPS tags that have invalid reference values
    let invalid_gps_tags: Vec<String> = output
        .iter()
        .filter(|(k, v)| {
            k.starts_with("GPS") && {
                if let Value::String(s) = v {
                    s.starts_with("Unknown") || s == "n/a"
                } else {
                    false
                }
            }
        })
        .map(|(k, _)| k.clone())
        .collect();
    for tag in invalid_gps_tags {
        output.remove(&tag);
    }

    // Wrap in an array like exiftool does
    Value::Array(vec![Value::Object(output)])
}
