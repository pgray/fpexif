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

/// Round a float value to 6 decimal places (for balance values like ExifTool)
fn round_balance_value(val: f64) -> f64 {
    (val * 1_000_000.0).round() / 1_000_000.0
}

/// Convert uppercase string to title case (e.g., "LOW" -> "Low", "WIDE ADAPTER" -> "Wide Adapter")
/// This matches ExifTool's formatting for certain Nikon MakerNote string values
fn to_title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().to_string() + &chars.as_str().to_lowercase(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
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

/// Format LensInfo (EXIF 0xA432 LensSpecification) in ExifTool format
/// Takes 4 rational values: (min_focal, max_focal, min_aperture, max_aperture)
/// Outputs format like "24-105mm f/0" or "50mm f/1.4" or "18-55mm f/?"
#[cfg(feature = "serde")]
fn format_lens_info(rationals: &[(u32, u32)]) -> Option<String> {
    if rationals.len() != 4 {
        return None;
    }

    // Convert rationals to f64, handling 0/0 as undefined
    let values: Vec<Option<f64>> = rationals
        .iter()
        .map(|(num, den)| {
            if *den == 0 {
                None // undefined
            } else {
                Some(*num as f64 / *den as f64)
            }
        })
        .collect();

    // Format focal length part
    // 0/0 = undefined = "?", 0/1 = 0 means explicitly zero = "0"
    let (min_focal, max_focal) = match (values[0], values[1]) {
        (Some(min), Some(max)) if min > 0.0 => {
            // Format as integers if they are whole numbers
            let min_str = if min.fract() == 0.0 {
                format!("{:.0}", min)
            } else {
                format!("{}", min)
            };
            if max > 0.0 && (max - min).abs() > 0.01 {
                let max_str = if max.fract() == 0.0 {
                    format!("{:.0}", max)
                } else {
                    format!("{}", max)
                };
                (min_str, Some(max_str))
            } else {
                (min_str, None)
            }
        }
        (Some(0.0), _) | (_, Some(0.0)) => {
            // Explicitly zero focal length - output "0"
            ("0".to_string(), None)
        }
        // Undefined (0/0) - output "?"
        _ => ("?".to_string(), None),
    };

    // Format aperture part
    let aperture_str = match (values[2], values[3]) {
        (None, _) | (_, None) => "?".to_string(), // undefined aperture
        (Some(min), Some(max)) => {
            if min == 0.0 {
                "0".to_string() // 0 means unknown
            } else if max > 0.0 && (max - min).abs() > 0.01 {
                // Range of apertures
                let min_str = if min.fract() == 0.0 {
                    format!("{:.0}", min)
                } else {
                    format!("{:.1}", min)
                };
                let max_str = if max.fract() == 0.0 {
                    format!("{:.0}", max)
                } else {
                    format!("{:.1}", max)
                };
                format!("{}-{}", min_str, max_str)
            } else {
                // Single aperture
                if min.fract() == 0.0 {
                    format!("{:.0}", min)
                } else {
                    format!("{:.1}", min)
                }
            }
        }
    };

    // Combine
    let focal_str = match max_focal {
        Some(max) => format!("{}-{}", min_focal, max),
        None => min_focal,
    };

    Some(format!("{}mm f/{}", focal_str, aperture_str))
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
        0x7000 => Value::String(crate::tags::get_sony_raw_file_type_description(value).to_string()),
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
        // ApertureValue and MaxApertureValue use uppercase "Inf"
        return if num == 0 {
            Value::String("undef".to_string())
        } else if tag_id == 0x9202 || tag_id == 0x9205 {
            Value::String("Inf".to_string())
        } else {
            Value::String("inf".to_string())
        };
    }

    match tag_id {
        0x829A => {
            // ExposureTime - ExifTool outputs:
            // - Decimal for times >= 0.25s (0.3, 0.5, 1, 2, 4.8, etc.)
            // - Fraction 1/n for fast shutter speeds < 0.25s
            let exposure_time = num as f64 / den as f64;
            if exposure_time >= 0.25001 {
                // For exposures >= ~1/4 second, use decimal format
                let rounded = (exposure_time * 10.0).round() / 10.0;
                if rounded == rounded.trunc() {
                    // Whole number - output as integer
                    Value::Number((rounded as i64).into())
                } else {
                    serde_json::Number::from_f64(rounded)
                        .map(Value::Number)
                        .unwrap_or_else(|| Value::Number((exposure_time.round() as i64).into()))
                }
            } else if exposure_time > 0.0 {
                // For sub-1/4 second exposures, use fraction format
                let approx_den = (1.0 / exposure_time).round() as u32;
                Value::String(format!("1/{}", approx_den))
            } else {
                Value::Number(0.into())
            }
        }
        0x9201 => {
            // ShutterSpeedValue (APEX) - convert to shutter speed
            // APEX: Tv = log2(1/t) where t is exposure time in seconds
            // So: t = 1/2^Tv = 2^(-Tv)
            // When Tv is positive, t < 1s (fast shutter, show as 1/N)
            // When Tv is negative, t > 1s (slow shutter, show as decimal)
            let apex_value = num as f64 / den as f64;
            let exposure_time = 2f64.powf(-apex_value);

            if exposure_time >= 0.25001 {
                // Slow shutter - output as decimal like ExifTool
                let rounded = (exposure_time * 10.0).round() / 10.0;
                if rounded == rounded.trunc() {
                    // Whole number
                    Value::Number((rounded as i64).into())
                } else {
                    serde_json::Number::from_f64(rounded)
                        .map(Value::Number)
                        .unwrap_or_else(|| Value::Number((rounded as i64).into()))
                }
            } else {
                // Fast shutter - output as fraction 1/N
                let denominator = (1.0 / exposure_time).round() as u32;
                Value::String(format!("1/{}", denominator))
            }
        }
        0x9202 | 0x9205 => {
            // ApertureValue, MaxApertureValue (APEX) - convert to f-number
            let apex_value = num as f64 / den as f64;
            let f_number = 2f64.powf(apex_value / 2.0);
            // ExifTool uses uppercase "Inf" for infinity
            if f_number.is_infinite() {
                Value::String("Inf".to_string())
            } else {
                let rounded = (f_number * 10.0).round() / 10.0;
                serde_json::Number::from_f64(rounded)
                    .map(Value::Number)
                    .unwrap_or_else(|| Value::String(rounded.to_string()))
            }
        }
        0x829D => {
            // FNumber - format depends on source
            let f_number = num as f64 / den as f64;
            // CRW files store FNumber with denominator 1000 (high precision from APEX value)
            // ExifTool uses %.2g format for Canon ShotInfo FNumber
            // CR2/standard EXIF uses %.1f format via PrintFNumber
            if den == 1000 {
                // CRW source - use %.2g equivalent (2 significant figures)
                // Values >= 10: round to integer (10.37 -> 10)
                // Values < 10: keep 1 decimal (5.6 -> 5.6)
                if f_number >= 10.0 {
                    Value::Number(serde_json::Number::from(f_number.round() as i64))
                } else {
                    let rounded = (f_number * 10.0).round() / 10.0;
                    Value::Number(
                        serde_json::Number::from_f64(rounded)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    )
                }
            } else {
                // Standard EXIF - use %.1f format
                let rounded = (f_number * 10.0).round() / 10.0;
                // Output integer when whole number (8 not 8.0), decimal otherwise
                if rounded.fract() == 0.0 {
                    Value::Number(serde_json::Number::from(rounded as i64))
                } else {
                    Value::Number(
                        serde_json::Number::from_f64(rounded)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    )
                }
            }
        }
        0x920A => {
            // FocalLength - add mm unit with decimal formatting
            // ExifTool uses "%.1f mm" format
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
        // ApertureValue and MaxApertureValue use uppercase "Inf"
        return if num == 0 {
            Value::String("undef".to_string())
        } else if tag_id == 0x9202 || tag_id == 0x9205 {
            Value::String("Inf".to_string())
        } else {
            Value::String("inf".to_string())
        };
    }

    match tag_id {
        0x9400 => {
            // AmbientTemperature - output with " C" suffix
            let temp = num as f64 / den as f64;
            // Use one decimal place like ExifTool (e.g., "27 C" or "27.5 C")
            // Preserve negative sign for -0 (when num=0 but den<0, or num<0 and result rounds to 0)
            if temp.fract() == 0.0 {
                let int_val = temp as i32;
                if int_val == 0 && (num < 0 || den < 0) && !(num < 0 && den < 0) {
                    // Negative zero: one operand is negative but not both (XOR)
                    Value::String("-0 C".to_string())
                } else {
                    Value::String(format!("{} C", int_val))
                }
            } else {
                Value::String(format!("{:.1} C", temp))
            }
        }
        0x9201 => {
            // ShutterSpeedValue (APEX) - convert to shutter speed
            // APEX: Tv = log2(1/t) where t is exposure time in seconds
            // So: t = 1/2^Tv = 2^(-Tv)
            // When Tv is positive, t < 1s (fast shutter, show as 1/N)
            // When Tv is negative, t > 1s (slow shutter, show as decimal)
            let apex_value = num as f64 / den as f64;
            let exposure_time = 2f64.powf(-apex_value);

            if exposure_time >= 0.25001 {
                // Slow shutter - output as decimal like ExifTool
                let rounded = (exposure_time * 10.0).round() / 10.0;
                if rounded == rounded.trunc() {
                    // Whole number
                    Value::Number((rounded as i64).into())
                } else {
                    serde_json::Number::from_f64(rounded)
                        .map(Value::Number)
                        .unwrap_or_else(|| Value::Number((rounded as i64).into()))
                }
            } else {
                // Fast shutter - output as fraction 1/N
                let denominator = (1.0 / exposure_time).round() as i32;
                Value::String(format!("1/{}", denominator))
            }
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
        // Nikon ExposureDifference (0x000E) - uses decimal format "%+.1f"
        // ExifTool: PrintConv => '$val ? sprintf("%+.1f",$val) : 0'
        0x000E => {
            let decimal = num as f64 / den as f64;
            let rounded = round_half_even(decimal, 1);
            if rounded == 0.0 {
                Value::Number(0.into())
            } else {
                // Format as decimal with 1 decimal place and + sign for positive
                Value::String(format!("{:+.1}", rounded))
            }
        }
        // Nikon MakerNote exposure-related tags (packed rational decode)
        // ProgramShift (0x000D), FlashExposureComp (0x0012),
        // ExternalFlashExposureComp (0x0017), FlashExposureBracketValue (0x0018), ExposureTuning (0x001C)
        // ExifTool outputs: 0 as int (except FlashExposureBracketValue which is 0.0),
        // negative as number (int for whole numbers), positive as string with "+" (fraction for 1/3 steps)
        0x000D | 0x0012 | 0x0017 | 0x0018 | 0x001C => {
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
                // Negative: use improper fraction format for common EV steps (ExifTool uses -5/3 not -1 2/3)
                let frac = rounded.fract().abs();
                let whole_abs = rounded.trunc().abs() as i32;
                if (frac - 0.3).abs() < 0.05 {
                    // x/3 fraction (1/3 step)
                    let numerator = whole_abs * 3 + 1;
                    Value::String(format!("-{}/3", numerator))
                } else if (frac - 0.7).abs() < 0.05 {
                    // x/3 fraction (2/3 step)
                    let numerator = whole_abs * 3 + 2;
                    Value::String(format!("-{}/3", numerator))
                } else if (frac - 0.5).abs() < 0.05 {
                    // x/2 fraction (1/2 step)
                    let numerator = whole_abs * 2 + 1;
                    Value::String(format!("-{}/2", numerator))
                } else if rounded.fract() == 0.0 {
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
                        let has_le_bom =
                            content.len() >= 2 && content[0] == 0xFF && content[1] == 0xFE;
                        let has_be_bom =
                            content.len() >= 2 && content[0] == 0xFE && content[1] == 0xFF;
                        let start = if has_le_bom || has_be_bom { 2 } else { 0 };

                        // Detect endianness: if no BOM, check if data looks like BE
                        // (first byte null, second byte ASCII) or LE (first byte ASCII, second null)
                        let is_be = if has_be_bom {
                            true
                        } else if has_le_bom {
                            false
                        } else if content.len() >= 2 {
                            // Heuristic: if first byte is 0 and second is ASCII, it's BE
                            content[0] == 0 && content[1] > 0 && content[1] < 128
                        } else {
                            false // default to LE
                        };

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
            Value::String(crate::tags::get_file_source_description(data[0]))
        }
        0xA301 if data.len() == 1 => {
            // SceneType
            Value::String(crate::tags::get_scene_type_description(data[0]))
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
        // ContrastCurve (MakerNote 0x008C) - always output as binary data message
        0x008C => Value::String(format_binary_data(data)),
        // TIFF-EPStandardID (0x9216) - when stored as undef, ExifTool outputs raw bytes
        // (only format as "1.0.0.0" when it's stored as int8u/Byte type, handled separately)
        // So for undefined type, we should NOT format it - let it fall through to raw output
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

            let cleaned: std::borrow::Cow<str> = if is_olympus_internal_serial {
                // Only trim null bytes, keep trailing spaces
                std::borrow::Cow::Borrowed(s.trim_end_matches('\0'))
            } else {
                // Remove ALL null bytes (not just ends), then trim whitespace
                // This handles EXIF Copyright format "Artist\0Editor\0" where both may be empty
                let without_nulls: String = s.chars().filter(|&c| c != '\0').collect();
                std::borrow::Cow::Owned(without_nulls.trim().to_string())
            };
            // If string is empty after trimming, return empty string
            if cleaned.is_empty() {
                return Value::String(String::new());
            }
            // FirmwareVersion should stay as string even if it looks numeric (e.g., "1.00")
            // Otherwise ExifTool JSON outputs "1.0" instead of "1.00"
            let is_firmware_version = tag_name == Some("FirmwareVersion");
            // ExifTool outputs pure numeric strings (like SerialNumber, SubSecTime)
            // as JSON numbers, so we do the same
            if !is_firmware_version && is_numeric_string(&cleaned) {
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
            0xA300 => Value::String(crate::tags::get_file_source_description(v[0])),
            0xA301 => Value::String(crate::tags::get_scene_type_description(v[0])),
            // GPSAltitudeRef (0x0005 in GPS IFD)
            0x0005 => {
                Value::String(crate::tags::get_gps_altitude_ref_description(v[0]).to_string())
            }
            _ => Value::Number(v[0].into()),
        },

        // Single-value Short - use helper function with make for manufacturer-specific decoding
        ExifValue::Short(v) if v.len() == 1 => {
            // Fujifilm AutoDynamicRange: format as "value%"
            if tag_name == Some("AutoDynamicRange") {
                return Value::String(format!("{}%", v[0]));
            }
            format_short_value_with_make(v[0], tag_id, make)
        }

        // Single-value other numeric types
        ExifValue::Long(v) if v.len() == 1 => {
            // Canon CR2 tags in IFD3
            match tag_id {
                0xc5e0 => {
                    // CR2CFAPattern
                    let pattern = match v[0] {
                        1 => "[Red,Green][Green,Blue]",
                        2 => "[Blue,Green][Green,Red]",
                        3 => "[Green,Blue][Red,Green]",
                        4 => "[Green,Red][Blue,Green]",
                        _ => return Value::Number(v[0].into()),
                    };
                    Value::String(pattern.to_string())
                }
                0xc6c5 => {
                    // SRawType - ExifTool outputs raw numeric value
                    Value::Number(v[0].into())
                }
                // Compression and ResolutionUnit can be stored as Long in some files (e.g., NRW)
                0x0103 => {
                    Value::String(crate::tags::get_compression_description(v[0] as u16).to_string())
                }
                0x0128 => Value::String(
                    crate::tags::get_resolution_unit_description(v[0] as u16).to_string(),
                ),
                _ => Value::Number(v[0].into()),
            }
        }
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

        // LensInfo/LensSpecification (0xA432) and DNGLensInfo (0xC630) - format as "24-105mm f/0"
        ExifValue::Rational(v) if (tag_id == 0xA432 || tag_id == 0xC630) && v.len() == 4 => {
            if let Some(formatted) = format_lens_info(v) {
                Value::String(formatted)
            } else {
                // Fallback to raw values if formatting fails
                let decimals: Vec<String> = v
                    .iter()
                    .map(|&(num, den)| {
                        if den == 0 {
                            "0".to_string()
                        } else {
                            format_float_10_sig(num as f64 / den as f64)
                        }
                    })
                    .collect();
                Value::String(decimals.join(" "))
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
            // RawImageDigest (0xC71C) - format as lowercase hex string
            } else if tag_id == 0xC71C {
                Value::String(v.iter().map(|b| format!("{:02x}", b)).collect::<String>())
            // RawDataUniqueID (0xC65D) - format as uppercase hex string
            } else if tag_id == 0xC65D {
                Value::String(v.iter().map(|b| format!("{:02X}", b)).collect::<String>())
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
                        // ApertureValue and MaxApertureValue use uppercase "Inf"
                        if num == 0 {
                            "undef".to_string()
                        } else if tag_id == 0x9202 || tag_id == 0x9205 {
                            "Inf".to_string()
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
                        // ApertureValue and MaxApertureValue use uppercase "Inf"
                        if num == 0 {
                            "undef".to_string()
                        } else if tag_id == 0x9202 || tag_id == 0x9205 {
                            "Inf".to_string()
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

    // Extract Model for model-specific formatting (tag 0x0110)
    let model: Option<String> = exif_data.get_tag_by_id(0x0110).and_then(|v| match v {
        ExifValue::Ascii(s) => Some(s.trim_end_matches('\0').trim().to_string()),
        _ => None,
    });
    let model_ref = model.as_deref();

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
        // Exception: SensingMethod (0xA217) from EXIF IFD should override Main IFD's 0x9217
        // ExifTool prefers EXIF IFD's SensingMethod over SubIFD's
        let should_override = tag_id.id == 0xA217; // SensingMethod in EXIF IFD
        if should_override || !output.contains_key(&tag_name) {
            let json_value = format_exif_value_for_json_with_make(value, tag_id.id, make_ref);
            output.insert(tag_name, json_value);
        }
    }

    // Add derived fields for exiftool compatibility
    // Aperture is derived from FNumber, rounded to 1 decimal place (%.1f equivalent)
    if let Some(ExifValue::Rational(v)) = exif_data.get_tag_by_id(0x829D) {
        if !v.is_empty() {
            // FNumber 0 means Inf (undefined aperture)
            if v[0].0 == 0 || v[0].1 == 0 {
                output.insert("Aperture".to_string(), Value::String("Inf".to_string()));
            } else {
                let aperture = v[0].0 as f64 / v[0].1 as f64;
                // ExifTool's PrintFNumber uses %.1f format
                let rounded = (aperture * 10.0).round() / 10.0;
                if let Some(num) = serde_json::Number::from_f64(rounded) {
                    output.insert("Aperture".to_string(), Value::Number(num));
                }
            }
        }
    }

    // ISO is an alias for ISOSpeedRatings (tag 0x8827)
    // Only use if not already set from MakerNotes (which may have Hi/Lo prefix)
    if !output.contains_key("ISO") {
        if let Some(ExifValue::Short(v)) = exif_data.get_tag_by_id(0x8827) {
            if !v.is_empty() {
                output.insert("ISO".to_string(), Value::Number(v[0].into()));
            }
        }
    }

    // SubfileType - derived from NewSubfileType (tag 0x00FE)
    // ExifTool calls this "SubfileType" (not "NewSubfileType")
    if let Some(ExifValue::Long(v)) = exif_data.get_tag_by_id(0x00FE) {
        if !v.is_empty() {
            let subfile_str = match v[0] {
                0 => "Full-resolution image",
                1 => "Reduced-resolution image",
                2 => "Single page of multi-page image",
                3 => "Single page of multi-page reduced-resolution image",
                4 => "Transparency mask",
                5 => "Transparency mask of reduced-resolution image",
                6 => "Transparency mask of multi-page image",
                7 => "Transparency mask of reduced-resolution multi-page image",
                8 => "Depth map",
                9 => "Depth map of reduced-resolution image",
                16 => "Enhanced image data",
                0x10001 => "Alternate reduced-resolution image",
                0x10004 => "Semantic Mask",
                _ => "Unknown",
            };
            output.insert(
                "SubfileType".to_string(),
                Value::String(subfile_str.to_string()),
            );
        }
    }
    // Also remove NewSubfileType if we have SubfileType
    output.remove("NewSubfileType");

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
    // For Sony: Only compute early for newer cameras (A6000+, A7+) that have SonyImageWidthMax
    // Older cameras (A100-A900) will have ImageSize computed later from FullImageSize

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

    // GPSTimeStamp (0x0007 in GPS IFD) - format as HH:MM:SS
    // Find GPSTimeStamp tag (id 0x0007 with 3 rationals) and format properly
    // Some cameras store this as SRational instead of Rational
    for (tag_id, value) in exif_data.iter() {
        if tag_id.id == 0x0007 {
            match value {
                ExifValue::Rational(time) if time.len() >= 3 => {
                    let formatted = format_gps_timestamp(time);
                    output.insert("GPSTimeStamp".to_string(), Value::String(formatted));
                    break;
                }
                ExifValue::SRational(time) if time.len() >= 3 => {
                    // Convert signed to unsigned for formatting
                    let unsigned: Vec<(u32, u32)> = time
                        .iter()
                        .map(|&(n, d)| (n.unsigned_abs(), d.unsigned_abs()))
                        .collect();
                    let formatted = format_gps_timestamp(&unsigned);
                    output.insert("GPSTimeStamp".to_string(), Value::String(formatted));
                    break;
                }
                _ => {}
            }
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
            // ExifTool rounds to 1 decimal, then uses integer if whole number
            let rounded = (altitude * 10.0).round() / 10.0;
            let alt_str = if rounded.fract() == 0.0 {
                format!("{:.0}", rounded)
            } else {
                format!("{:.1}", rounded)
            };
            // ExifTool defaults unknown altitude ref to "Above Sea Level" for non-negative values
            let formatted = if let Some(ref_str) = ref_desc {
                if ref_str != "Unknown" {
                    format!("{} m {}", alt_str, ref_str)
                } else if altitude < 0.0 {
                    format!("{} m Below Sea Level", alt_str.trim_start_matches('-'))
                } else {
                    format!("{} m Above Sea Level", alt_str)
                }
            } else if altitude < 0.0 {
                format!("{} m Below Sea Level", alt_str.trim_start_matches('-'))
            } else {
                format!("{} m Above Sea Level", alt_str)
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

    // GPSMeasureMode (0x000A) - expand to human-readable description
    if let Some(ExifValue::Ascii(mode)) = exif_data.get_tag_by_id(0x000A) {
        let expanded = match mode.trim() {
            "2" => "2-Dimensional Measurement",
            "3" => "3-Dimensional Measurement",
            other => other,
        };
        output.insert(
            "GPSMeasureMode".to_string(),
            Value::String(expanded.to_string()),
        );
    }

    // GPSSpeedRef (0x000C) - expand to human-readable units
    if let Some(ExifValue::Ascii(ref_val)) = exif_data.get_tag_by_id(0x000C) {
        let trimmed = ref_val.trim();
        let expanded = match trimmed {
            "K" => "km/h".to_string(),
            "M" => "mph".to_string(),
            "N" => "knots".to_string(),
            "" => "Unknown ()".to_string(),
            other => format!("Unknown ({})", other),
        };
        output.insert("GPSSpeedRef".to_string(), Value::String(expanded));
    }

    // GPSTrackRef (0x000E) - expand to North reference type
    if let Some(ExifValue::Ascii(ref_val)) = exif_data.get_tag_by_id(0x000E) {
        let trimmed = ref_val.trim();
        let expanded = match trimmed {
            "M" => "Magnetic North".to_string(),
            "T" => "True North".to_string(),
            "" => "Unknown ()".to_string(),
            other => format!("Unknown ({})", other),
        };
        output.insert("GPSTrackRef".to_string(), Value::String(expanded));
    }

    // GPSImgDirectionRef (0x0010) - expand to North reference type
    if let Some(ExifValue::Ascii(ref_val)) = exif_data.get_tag_by_id(0x0010) {
        let trimmed = ref_val.trim();
        let expanded = match trimmed {
            "M" => "Magnetic North".to_string(),
            "T" => "True North".to_string(),
            "" => "Unknown ()".to_string(),
            other => format!("Unknown ({})", other),
        };
        output.insert("GPSImgDirectionRef".to_string(), Value::String(expanded));
    }

    // GPSDestLatitudeRef (0x0013) - expand N/S to North/South
    if let Some(ExifValue::Ascii(ref_val)) = exif_data.get_tag_by_id(0x0013) {
        let trimmed = ref_val.trim();
        if !trimmed.is_empty() {
            let expanded = crate::tags::get_gps_latitude_ref_description(trimmed);
            output.insert(
                "GPSDestLatitudeRef".to_string(),
                Value::String(expanded.to_string()),
            );
        }
    }

    // GPSDestLongitudeRef (0x0015) - expand E/W to East/West
    if let Some(ExifValue::Ascii(ref_val)) = exif_data.get_tag_by_id(0x0015) {
        let trimmed = ref_val.trim();
        if !trimmed.is_empty() {
            let expanded = crate::tags::get_gps_longitude_ref_description(trimmed);
            output.insert(
                "GPSDestLongitudeRef".to_string(),
                Value::String(expanded.to_string()),
            );
        }
    }

    // GPSDestBearingRef (0x0017) - expand to North reference type
    if let Some(ExifValue::Ascii(ref_val)) = exif_data.get_tag_by_id(0x0017) {
        let trimmed = ref_val.trim();
        let expanded = match trimmed {
            "M" => "Magnetic North".to_string(),
            "T" => "True North".to_string(),
            "" => "Unknown ()".to_string(),
            other => format!("Unknown ({})", other),
        };
        output.insert("GPSDestBearingRef".to_string(), Value::String(expanded));
    }

    // GPSDestDistanceRef (0x0019) - expand to distance units
    if let Some(ExifValue::Ascii(ref_val)) = exif_data.get_tag_by_id(0x0019) {
        let trimmed = ref_val.trim();
        let expanded = match trimmed {
            "K" => "Kilometers".to_string(),
            "M" => "Miles".to_string(),
            "N" => "Nautical Miles".to_string(),
            "" => "Unknown ()".to_string(),
            other => format!("Unknown ({})", other),
        };
        output.insert("GPSDestDistanceRef".to_string(), Value::String(expanded));
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

    // OffsetTime tags for timezone info (EXIF 2.31+)
    // 0x9010: OffsetTime (for ModifyDate)
    // 0x9011: OffsetTimeOriginal (for DateTimeOriginal)
    // 0x9012: OffsetTimeDigitized (for CreateDate/DateTimeDigitized)
    let offset_time = exif_data.get_tag_by_id(0x9010).and_then(|v| match v {
        ExifValue::Ascii(s) => Some(s.trim_end_matches('\0').trim().to_string()),
        _ => None,
    });
    let offset_time_original = exif_data.get_tag_by_id(0x9011).and_then(|v| match v {
        ExifValue::Ascii(s) => Some(s.trim_end_matches('\0').trim().to_string()),
        _ => None,
    });
    let offset_time_digitized = exif_data.get_tag_by_id(0x9012).and_then(|v| match v {
        ExifValue::Ascii(s) => Some(s.trim_end_matches('\0').trim().to_string()),
        _ => None,
    });

    // SubSecModifyDate = ModifyDate (0x0132) + SubSecTime (0x9290) + OffsetTime (0x9010)
    if let (Some(ExifValue::Ascii(date)), Some(subsec)) =
        (exif_data.get_tag_by_id(0x0132), &subsec_time)
    {
        let cleaned = date.trim_end_matches('\0').trim();
        let tz = offset_time.as_deref().unwrap_or("");
        output.insert(
            "SubSecModifyDate".to_string(),
            Value::String(format!("{}.{}{}", cleaned, subsec, tz)),
        );
    }

    // SubSecDateTimeOriginal = DateTimeOriginal (0x9003) + SubSecTimeOriginal (0x9291) + OffsetTimeOriginal (0x9011)
    if let (Some(ExifValue::Ascii(date)), Some(subsec)) =
        (exif_data.get_tag_by_id(0x9003), &subsec_original)
    {
        let cleaned = date.trim_end_matches('\0').trim();
        let tz = offset_time_original.as_deref().unwrap_or("");
        output.insert(
            "SubSecDateTimeOriginal".to_string(),
            Value::String(format!("{}.{}{}", cleaned, subsec, tz)),
        );
    }

    // SubSecCreateDate = DateTimeDigitized (0x9004) + SubSecTimeDigitized (0x9292) + OffsetTimeDigitized (0x9012)
    if let (Some(ExifValue::Ascii(date)), Some(subsec)) =
        (exif_data.get_tag_by_id(0x9004), &subsec_digitized)
    {
        let cleaned = date.trim_end_matches('\0').trim();
        let tz = offset_time_digitized.as_deref().unwrap_or("");
        output.insert(
            "SubSecCreateDate".to_string(),
            Value::String(format!("{}.{}{}", cleaned, subsec, tz)),
        );
    }

    // Add maker notes if present
    // MakerNote tags can override EXIF tags for certain fields where the MakerNote
    // value is more accurate (like MeteringMode for Canon cameras)
    if let Some(maker_notes) = exif_data.get_maker_notes() {
        // Tags where MakerNote should override EXIF (ExifTool behavior)
        // FocusMode added: Sony has multiple FocusMode tags (0x201B, 0xB042, 0xB04E) where
        // higher IDs are used for newer cameras and should take precedence
        // Note: Saturation/Contrast/Sharpness handled separately below for Pentax/Samsung
        // where MakerNote format differs (e.g., "0 (normal)" vs standard "Normal")
        // WhiteLevel: Sony SR2SubIFD WhiteLevel (3 values) should override main IFD WhiteLevel (1 value)
        const MAKERNOTE_PRIORITY_TAGS: &[&str] = &[
            "MeteringMode",
            "WhiteBalance",
            "LightSource",
            "FocusMode",
            "WhiteLevel",
        ];

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
            // Also allow Sony RIF sub-tags (0xD001-0xD003) from MRWInfo for A100
            let is_sony_camera_settings_subtag = (*tag_id >= 0xC01C && *tag_id <= 0xC01E)
                || (*tag_id >= 0xD001 && *tag_id <= 0xD003);
            let is_sony_subtag =
                is_sony_camera_settings_subtag && make_ref.is_some_and(|m| m.contains("SONY"));

            // For Pentax/Samsung cameras in native formats (PEF, SRW), Saturation/Contrast/
            // Sharpness should use standard EXIF values (0xA408/0xA409/0xA40A). ExifTool
            // outputs "Normal", "Hard", "Soft" from EXIF, not Pentax format like "+1 (medium hard)"
            // However, for DNG files with Pentax MakerNotes, ExifTool uses MakerNote values.
            let is_pentax_or_samsung = make_ref.is_some_and(|m| {
                let upper = m.to_uppercase();
                upper.contains("PENTAX") || upper.contains("SAMSUNG") || upper.contains("RICOH")
            });
            let is_dng_file = source_file
                .map(|f| f.to_uppercase().ends_with(".DNG"))
                .unwrap_or(false);
            let is_scs_tag =
                tag_name == "Saturation" || tag_name == "Contrast" || tag_name == "Sharpness";
            let is_pentax_skip_tag = is_pentax_or_samsung && !is_dng_file && is_scs_tag;

            // Skip Pentax/Samsung Saturation/Contrast/Sharpness in non-DNG files
            if is_pentax_skip_tag {
                continue;
            }

            // For DNG files with Pentax MakerNotes, these tags should override EXIF
            let is_pentax_dng_override = is_pentax_or_samsung && is_dng_file && is_scs_tag;

            // For Minolta cameras, MeteringMode should NOT override EXIF - ExifTool uses
            // the standard EXIF MeteringMode (0x9207) not the CameraSettings one
            let is_minolta = make_ref.is_some_and(|m| {
                let upper = m.to_uppercase();
                upper.contains("MINOLTA") || upper.contains("KONICA")
            });
            let is_minolta_metering_skip = is_minolta && tag_name == "MeteringMode";

            // For Kodak cameras, MeteringMode should NOT override EXIF - ExifTool uses
            // the standard EXIF MeteringMode (0x9207) not the MakerNote one
            let is_kodak = make_ref.is_some_and(|m| m.to_uppercase().contains("KODAK"));
            let is_kodak_metering_skip = is_kodak && tag_name == "MeteringMode";

            // For Nikon, SerialNumber tag 0x00A0 should override 0x001D because
            // 0x001D often contains model name on older cameras, not serial number
            let is_nikon = make_ref.is_some_and(|m| m.to_uppercase().contains("NIKON"));
            let is_nikon_serial_override =
                is_nikon && tag_name == "SerialNumber" && *tag_id == 0x00A0;

            let should_insert = if is_minolta_metering_skip || is_kodak_metering_skip {
                false // Skip MakerNote MeteringMode, use EXIF instead
            } else if is_nikon_serial_override
                || MAKERNOTE_PRIORITY_TAGS.contains(&tag_name)
                || is_sony_subtag
                || is_pentax_dng_override
            {
                // Nikon SerialNumber 0x00A0 always overrides 0x001D
                // Priority tags always use MakerNote value
                true
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
            } else if let Some(Value::String(lens_model)) = output.get("LensModel") {
                // Prefer LensModel when it contains detailed lens info (e.g., "FE 70-200mm F2.8 GM OSS II")
                // LensModel often has more complete info than LensSpec
                if lens_model.contains("mm") && lens_model.contains("F") {
                    let lens_id = if (lens_model.starts_with("E ")
                        || lens_model.starts_with("FE ")
                        || lens_model.starts_with("DT "))
                        && !lens_model.contains("Tamron")
                        && !lens_model.contains("Sigma")
                        && !lens_model.contains("Zeiss")
                        && !lens_model.contains("Samyang")
                        && !lens_model.contains("Voigtlander")
                    {
                        format!("Sony {}", lens_model)
                    } else {
                        lens_model.clone()
                    };
                    output.insert("LensID".to_string(), Value::String(lens_id));
                }
            }
            // Fallback to LensSpec if LensID still not set
            // Skip LensSpec if it's "Unknown..." (no lens attached or unrecognized)
            if !output.contains_key("LensID")
                || matches!(output.get("LensID"), Some(Value::String(s)) if s.contains("E-Mount"))
            {
                if let Some(Value::String(lens_spec)) = output.get("LensSpec") {
                    // Only use LensSpec if it's a valid lens specification
                    if !lens_spec.starts_with("Unknown") {
                        // Use LensSpec for E-mount lenses when LensType2 isn't available
                        // Add "Sony " prefix only for Sony lenses (E/FE/DT prefix without brand name)
                        let lens_id = if (lens_spec.starts_with("E ")
                            || lens_spec.starts_with("FE ")
                            || lens_spec.starts_with("DT "))
                            && !lens_spec.contains("Tamron")
                            && !lens_spec.contains("Sigma")
                            && !lens_spec.contains("Zeiss")
                            && !lens_spec.contains("Samyang")
                            && !lens_spec.contains("Voigtlander")
                        {
                            format!("Sony {}", lens_spec)
                        } else {
                            lens_spec.clone()
                        };
                        output.insert("LensID".to_string(), Value::String(lens_id));
                    }
                }
            }
            // Fallback to LensType - use it even for E-Mount placeholder if no better option
            if !output.contains_key("LensID") {
                if let Some(lens_type) = output.get("LensType").cloned() {
                    output.insert("LensID".to_string(), lens_type);
                }
            }
        }
    } else if !output.contains_key("LensID") {
        if is_canon || is_olympus {
            // Canon: First check RFLensType for RF mount lenses - this gives specific lens ID
            if is_canon {
                if let Some(Value::String(rf_lens_type)) = output.get("RFLensType") {
                    if rf_lens_type != "n/a" && rf_lens_type != "Unknown" {
                        output.insert("LensID".to_string(), Value::String(rf_lens_type.clone()));
                    }
                }
            }

            // Canon/Olympus: LensID should come from LensType, but disambiguated using LensModel
            // If LensType has "or ..." suffix and LensModel matches the Canon lens, remove the suffix
            // Skip if LensID was already set from RFLensType above
            if !output.contains_key("LensID") {
                if let Some(Value::String(lens_type)) = output.get("LensType") {
                    let lens_id = if lens_type.contains(" or ") {
                        // Try to disambiguate using LensModel
                        if let Some(Value::String(lens_model)) = output.get("LensModel") {
                            // LensModel has format like "EF17-40mm f/4L USM" - normalize and compare
                            let model_normalized = lens_model
                                .replace("EF-S", "EF-S ")
                                .replace("EF", "EF ")
                                .replace("  ", " ")
                                .trim()
                                .to_string();

                            // Extract Canon lens name (before " or ")
                            let canon_part = lens_type.split(" or ").next().unwrap_or(lens_type);

                            // Check if the model matches the Canon lens (focal lengths and aperture)
                            let model_matches = {
                                // Extract key parts: focal length and aperture
                                let model_has_match =
                                    model_normalized.contains("f/") || lens_model.contains("f/");
                                let type_has_match = canon_part.contains("f/");

                                // If both have aperture info, they should match
                                if model_has_match && type_has_match {
                                    // Simple check: if LensModel starts with EF/EF-S and
                                    // contains similar focal length info, it's likely a Canon lens
                                    lens_model.starts_with("EF")
                                } else {
                                    // Fall back to checking if it looks like a Canon lens
                                    lens_model.starts_with("EF")
                                }
                            };

                            if model_matches {
                                // Use Canon part only (without " or ..." suffix)
                                Value::String(canon_part.to_string())
                            } else {
                                Value::String(lens_type.clone())
                            }
                        } else {
                            Value::String(lens_type.clone())
                        }
                    } else {
                        Value::String(lens_type.clone())
                    };
                    output.insert("LensID".to_string(), lens_id);
                }
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
                // Panasonic/Leica: Format lens name by adding "mm" before /F
                // e.g., "LUMIX G VARIO 14-45/F3.5-5.6" -> "LUMIX G VARIO 14-45mm F3.5-5.6"
                let is_panasonic = make_ref
                    .map(|m| {
                        let upper = m.to_uppercase();
                        upper.contains("PANASONIC") || upper.contains("LEICA")
                    })
                    .unwrap_or(false);
                let lens_id = if is_panasonic {
                    if let Value::String(s) = &lens_type {
                        // Replace "/F" with "mm F" (e.g., "14-45/F3.5-5.6" -> "14-45mm F3.5-5.6")
                        Value::String(s.replace("/F", "mm F"))
                    } else {
                        lens_type
                    }
                } else {
                    lens_type
                };
                output.insert("LensID".to_string(), lens_id);
            }
        }
    }

    // Canon DriveMode composite - computed from ContinuousDrive and SelfTimer
    // Based on ExifTool's Canon::Composite DriveMode
    if is_canon && !output.contains_key("DriveMode") {
        let continuous = output.get("ContinuousDrive").and_then(|v| {
            if let Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        });
        let self_timer = output.get("SelfTimer").and_then(|v| {
            if let Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        });

        if let (Some(cont), Some(timer)) = (continuous, self_timer) {
            let drive_mode = if cont != "Single" && cont != "Off" {
                "Continuous Shooting"
            } else if timer != "Off" && timer != "0" && !timer.starts_with("0 ") {
                "Self-timer Operation"
            } else {
                "Single-frame Shooting"
            };
            output.insert(
                "DriveMode".to_string(),
                Value::String(drive_mode.to_string()),
            );
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

    // Ensure ImageWidth/ImageHeight are populated (ExifTool composite behavior)
    // Priority: manufacturer-specific (always override) > existing > ExifImageWidth/Height
    // Note: SonyImageWidth/Height are NOT used here - they're standalone tags, not replacements
    // for ImageWidth/Height (ExifTool uses different logic for Sony)

    // For Sony: Process FullImageSize (tag 0xb02b)
    // The tag is already formatted as "WxH" in sony.rs, we just need to extract dimensions
    // Note: Only older Sony cameras (A100-A900) use FullImageSize for ImageWidth/Height/ImageSize
    // Newer cameras (A6000+, A7+) use IFD0 ImageWidth/ImageLength instead
    let is_sony_camera = make_ref
        .map(|m| m.to_uppercase().contains("SONY"))
        .unwrap_or(false);
    if is_sony_camera {
        if let Some(Value::String(full_size)) = output.get("FullImageSize").cloned() {
            // FullImageSize is already formatted as "WxH" (e.g., "3872x2592") in sony.rs
            let parts: Vec<&str> = full_size.split('x').collect();
            if parts.len() == 2 {
                if let (Ok(width), Ok(height)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>())
                {
                    // Determine whether to use FullImageSize for ImageWidth/Height
                    // Early DSLR-A models (A100-A350, A700, A850, A900) use FullImageSize
                    // Later models (A550+, SLT, NEX, ILCE, DSC) use IFD0 dimensions
                    let model = output.get("Model").and_then(|v| match v {
                        Value::String(s) => Some(s.as_str()),
                        _ => None,
                    });
                    let has_sony_image_width = output.contains_key("SonyImageWidth");

                    // Use FullImageSize for specific early DSLR-A models
                    // These models have sensor overscan that needs cropping via FullImageSize
                    // Note: A450/A500/A550 use IFD0 dimensions directly (FullImageSize is different)
                    let is_early_dslr = model.is_some_and(|m| {
                        m.starts_with("DSLR-A")
                            && (m.contains("A100")
                                || m.contains("A200")
                                || m.contains("A230")
                                || m.contains("A290")
                                || m.contains("A300")
                                || m.contains("A330")
                                || m.contains("A350")
                                || m.contains("A380")
                                || m.contains("A390")
                                || m.contains("A700")
                                || m.contains("A850")
                                || m.contains("A900"))
                    });
                    let use_full_image_size = !has_sony_image_width && is_early_dslr;

                    if use_full_image_size {
                        output.insert("ImageWidth".to_string(), Value::Number(width.into()));
                        output.insert("ImageHeight".to_string(), Value::Number(height.into()));
                        output.insert(
                            "ImageSize".to_string(),
                            Value::String(format!("{}x{}", width, height)),
                        );
                        let mp = (width as f64 * height as f64) / 1_000_000.0;
                        let rounded = (mp * 10.0).round() / 10.0;
                        if let Some(num) = serde_json::Number::from_f64(rounded) {
                            output.insert("Megapixels".to_string(), Value::Number(num));
                        }
                    }
                }
            }
        }
    }

    // Check for manufacturer-specific width tags that OVERRIDE existing values
    let mfr_width = output
        .get("RawImageCroppedWidth")
        .or_else(|| output.get("PanasonicImageWidth"))
        .cloned();
    if let Some(w) = mfr_width {
        output.insert("ImageWidth".to_string(), w);
    } else if !output.contains_key("ImageWidth") {
        // Only fall back to ExifImageWidth if no ImageWidth exists
        if let Some(eiw) = output.get("ExifImageWidth").cloned() {
            output.insert("ImageWidth".to_string(), eiw);
        }
    }

    // Check for manufacturer-specific height tags that OVERRIDE existing values
    let mfr_height = output
        .get("RawImageCroppedHeight")
        .or_else(|| output.get("PanasonicImageHeight"))
        .cloned();
    if let Some(h) = mfr_height {
        output.insert("ImageHeight".to_string(), h);
    } else if !output.contains_key("ImageHeight") && !output.contains_key("ImageLength") {
        // Only fall back to ExifImageHeight if no ImageHeight exists
        if let Some(eih) = output.get("ExifImageHeight").cloned() {
            output.insert("ImageHeight".to_string(), eih);
        }
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
            } else if key == "RawExposureBias" || key == "RedBalance" || key == "BlueBalance" {
                // These should be numbers in JSON
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

    // Add MRW-specific metadata if present (RIF block for Minolta RAW files)
    // ExifTool behavior varies by model:
    // - DiMAGE cameras: ExifTool outputs raw numeric values from RIF block
    // - DYNAX/Alpha cameras: ExifTool uses EXIF IFD values with text descriptions
    let is_dimage = model_ref
        .map(|m| m.to_uppercase().contains("DIMAGE"))
        .unwrap_or(false);
    if let Some(mrw_metadata) = exif_data.get_mrw_metadata() {
        for (key, value) in &mrw_metadata.tags {
            // For Saturation, Contrast, Sharpness:
            // - DiMAGE: override with RIF block numeric values
            // - DYNAX/Alpha: keep EXIF values if present (already decoded)
            if key == "Saturation" || key == "Contrast" || key == "Sharpness" {
                if !is_dimage && output.contains_key(key) {
                    continue;
                }
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

    // Add RW2-specific metadata (PanasonicRaw IFD0 data)
    if let Some(rw2_metadata) = exif_data.get_rw2_metadata() {
        for (key, value) in &rw2_metadata.tags {
            // Try to parse as number first, then fall back to string
            if let Ok(n) = value.parse::<i64>() {
                output.insert(key.clone(), Value::Number(n.into()));
            } else if value.parse::<f64>().is_ok() {
                // For floating point values, store as string to preserve precision
                output.insert(key.clone(), Value::String(value.clone()));
            } else {
                output.insert(key.clone(), Value::String(value.clone()));
            }
        }

        // Compute RedBalance and BlueBalance from WB levels if not already present
        // ExifTool does: RedBalance = WBRedLevel / WBGreenLevel
        //               BlueBalance = WBBlueLevel / WBGreenLevel
        if !output.contains_key("RedBalance") || !output.contains_key("BlueBalance") {
            let wb_red = rw2_metadata
                .tags
                .get("WBRedLevel")
                .and_then(|v| v.parse::<f64>().ok());
            let wb_green = rw2_metadata
                .tags
                .get("WBGreenLevel")
                .and_then(|v| v.parse::<f64>().ok());
            let wb_blue = rw2_metadata
                .tags
                .get("WBBlueLevel")
                .and_then(|v| v.parse::<f64>().ok());

            if let (Some(r), Some(g), Some(b)) = (wb_red, wb_green, wb_blue) {
                if g > 0.0 {
                    if !output.contains_key("RedBalance") {
                        output.insert(
                            "RedBalance".to_string(),
                            Value::String(format!("{:.6}", r / g)),
                        );
                    }
                    if !output.contains_key("BlueBalance") {
                        output.insert(
                            "BlueBalance".to_string(),
                            Value::String(format!("{:.6}", b / g)),
                        );
                    }
                }
            }
        }
    }

    // Sony A100 Saturation/Contrast/Sharpness fix
    // ExifTool outputs raw numeric values (from MakerNotes) for the A100 instead of decoded text
    // A100 was the first Sony DSLR after acquiring Konica Minolta, shares similar behavior
    let is_sony_a100 = model_ref.map(|m| m.contains("DSLR-A100")).unwrap_or(false);
    if is_sony_a100 {
        // Check if we have the standard EXIF Saturation/Sharpness/Contrast decoded to "Normal"
        // and convert back to raw 0 to match ExifTool
        for key in ["Saturation", "Sharpness", "Contrast"] {
            if let Some(Value::String(s)) = output.get(key) {
                if s == "Normal" {
                    output.insert(key.to_string(), Value::Number(0.into()));
                }
            }
        }
    }

    // ImageSize - compute from actual image dimensions
    // Priority order varies by manufacturer:
    // - Fuji: RawImageCroppedWidth/Height from RAF metadata
    // - Panasonic: PanasonicImageWidth/Height from MakerNote
    // - Canon: ExifImageWidth/Height (IFD0 has thumbnail dimensions in CR2)
    // - Sony: ImageWidth/ImageLength from IFD0 (as fixed above)
    // - Others: ImageWidth/ImageLength, then ExifImageWidth/Height
    let is_canon = make_ref
        .map(|m| m.to_uppercase().contains("CANON"))
        .unwrap_or(false);

    let width = output
        .get("RawImageCroppedWidth")
        .or_else(|| output.get("PanasonicImageWidth"))
        .or_else(|| {
            // Canon CR2: Use ExifImageWidth (actual size) over IFD0 ImageWidth (thumbnail)
            if is_canon {
                output.get("ExifImageWidth")
            } else {
                output.get("ImageWidth")
            }
        })
        .or_else(|| {
            if is_canon {
                output.get("ImageWidth")
            } else {
                output.get("ExifImageWidth")
            }
        })
        .and_then(|v| match v {
            Value::Number(n) => n.as_u64(),
            _ => None,
        });
    let height = output
        .get("RawImageCroppedHeight")
        .or_else(|| output.get("PanasonicImageHeight"))
        .or_else(|| {
            if is_canon {
                output.get("ExifImageHeight")
            } else {
                output.get("ImageHeight")
            }
        })
        .or_else(|| {
            if is_canon {
                output.get("ImageHeight")
            } else {
                output.get("ImageLength")
            }
        })
        .or_else(|| output.get("ImageLength"))
        .or_else(|| output.get("ExifImageHeight"))
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
                let red_balance = round_balance_value(r / g);
                if let Some(num) = serde_json::Number::from_f64(red_balance) {
                    output.insert("RedBalance".to_string(), Value::Number(num));
                }
            }
            if !output.contains_key("BlueBalance") {
                let blue_balance = round_balance_value(b / g);
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
                        let red_balance = round_balance_value(r / g);
                        if let Some(num) = serde_json::Number::from_f64(red_balance) {
                            output.insert("RedBalance".to_string(), Value::Number(num));
                        }
                    }
                    if !output.contains_key("BlueBalance") {
                        let blue_balance = round_balance_value(b / g);
                        if let Some(num) = serde_json::Number::from_f64(blue_balance) {
                            output.insert("BlueBalance".to_string(), Value::Number(num));
                        }
                    }
                }
            }
        }
    }

    // Also try WB_RBLevels format (R B G1 G2) - common in Olympus
    // ExifTool uses fixed divisor of 256 for WB_RBLevels (see Exif.pm rggbLookup index 8)
    // RedBalance = R / 256, BlueBalance = B / 256
    if !output.contains_key("RedBalance") || !output.contains_key("BlueBalance") {
        if let Some(Value::String(wb_levels)) = output.get("WB_RBLevels") {
            let parts: Vec<f64> = wb_levels
                .split_whitespace()
                .filter_map(|s| s.parse::<f64>().ok())
                .collect();
            if parts.len() >= 2 {
                let r = parts[0];
                let b = parts[1];
                // ExifTool uses fixed 256 as divisor for RB format
                // (from rggbLookup index 8: [0, 256, 256, 1])
                let divisor = 256.0;
                if !output.contains_key("RedBalance") {
                    let red_balance = round_balance_value(r / divisor);
                    if let Some(num) = serde_json::Number::from_f64(red_balance) {
                        output.insert("RedBalance".to_string(), Value::Number(num));
                    }
                }
                if !output.contains_key("BlueBalance") {
                    let blue_balance = round_balance_value(b / divisor);
                    if let Some(num) = serde_json::Number::from_f64(blue_balance) {
                        output.insert("BlueBalance".to_string(), Value::Number(num));
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
    // For computed fields (HyperfocalDistance, DOF, LightValue), use raw FNumber rational
    // to maintain precision. The formatted FNumber may have reduced precision (e.g., %.2g for CRW).
    let aperture = exif_data
        .get_tag_by_id(0x829D)
        .and_then(|v| match v {
            ExifValue::Rational(r) if !r.is_empty() && r[0].1 != 0 => {
                Some(r[0].0 as f64 / r[0].1 as f64)
            }
            _ => None,
        })
        .or_else(|| output.get("FNumber").and_then(parse_f64_value))
        .or_else(|| output.get("Aperture").and_then(parse_f64_value));
    // For LightValue calculation, use raw rational for precision
    // Tag 0x829A (ExposureTime) is a rational, use raw value to avoid rounding
    let exposure_time = exif_data
        .get_tag_by_id(0x829A)
        .and_then(|v| match v {
            ExifValue::Rational(r) if !r.is_empty() && r[0].1 != 0 => {
                Some(r[0].0 as f64 / r[0].1 as f64)
            }
            _ => None,
        })
        .or_else(|| output.get("ExposureTime").and_then(parse_f64_value));
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
    // ExifTool formatting:
    // - < 0.25001s: string as "1/N" fraction
    // - >= 0.25001s: number (integer if whole, otherwise 1 decimal)
    if !has_shutter_speed {
        if let Some(exp_time) = exposure_time {
            if exp_time > 0.0 {
                if exp_time < 0.25001 {
                    // Format as fraction string "1/N"
                    let formatted = format!("1/{}", (0.5 + 1.0 / exp_time) as u32);
                    output.insert("ShutterSpeed".to_string(), Value::String(formatted));
                } else {
                    // Output as number - ExifTool outputs integers for whole seconds
                    let rounded = (exp_time * 10.0).round() / 10.0;
                    if rounded == rounded.trunc() {
                        // Whole number - output as integer
                        output.insert(
                            "ShutterSpeed".to_string(),
                            Value::Number(serde_json::Number::from(rounded as i64)),
                        );
                    } else {
                        // Has decimal - output as float
                        if let Some(n) = serde_json::Number::from_f64(rounded) {
                            output.insert("ShutterSpeed".to_string(), Value::Number(n));
                        }
                    }
                }
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
    // When base focal length is non-zero but 35mm equivalent would be 0
    // (unknown scale factor), just show the focal length without equivalent
    if !has_lens_35 {
        if let (Some(min), Some(max)) = (min_fl, max_fl) {
            let (min35, max35) = scale_factor
                .map(|sf| (min * sf, max * sf))
                .unwrap_or((0.0, 0.0));

            // Only skip 35mm equivalent if base focal length is valid but
            // scale factor is unknown (results in 0 equivalent)
            let skip_35_equiv = min > 0.1 && min35 < 0.1;

            let formatted = if (min - max).abs() < 0.01 {
                // Single focal length lens (prime)
                if skip_35_equiv {
                    format!("{:.1} mm", min)
                } else {
                    format!("{:.1} mm (35 mm equivalent: {:.1} mm)", min, min35)
                }
            } else {
                // Zoom lens
                if skip_35_equiv {
                    format!("{:.1} - {:.1} mm", min, max)
                } else {
                    format!(
                        "{:.1} - {:.1} mm (35 mm equivalent: {:.1} - {:.1} mm)",
                        min, max, min35, max35
                    )
                }
            };
            output.insert("Lens35efl".to_string(), Value::String(formatted));
        }
    }

    // CircleOfConfusion - sensor diagonal / 1440
    // Full frame diagonal = sqrt(24² + 36²) (matches ExifTool's formula)
    // CoC = sqrt(24*24 + 36*36) / (ScaleFactor × 1440)
    let full_frame_diagonal = (24.0_f64 * 24.0 + 36.0 * 36.0).sqrt();
    let coc = scale_factor.map(|sf| full_frame_diagonal / (sf * 1440.0));
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

                // ExifTool: atan2(36, 2*FocalLength*ScaleFactor*corr) * 360 / 3.14159
                // Use ExifTool's approximation of PI for exact match
                let half_fov = (36.0 / (2.0 * fl * sf * corr)).atan();
                #[allow(clippy::approx_constant)]
                let fov_deg = half_fov * 360.0 / 3.14159;

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
    // ExifTool formula: t = Aperture × CoC × (d×1000 - f) / f²
    //                   near = d / (1 + t), far = d / (1 - t)
    if !has_dof {
        if let Some(Value::String(s)) = focus_dist_str {
            if let Ok(focus_m) = s.trim_end_matches(" m").parse::<f64>() {
                if let (Some(fl), Some(ap), Some(c)) = (focal_length, aperture, coc) {
                    if ap > 0.0 && focus_m > 0.0 && focus_m.is_finite() && c > 0.0 {
                        // ExifTool formula (from Exif.pm)
                        let t = ap * c * (focus_m * 1000.0 - fl) / (fl * fl);
                        let near_m = focus_m / (1.0 + t);
                        let far_m = focus_m / (1.0 - t);

                        // If far_m is negative or t >= 1, far is infinity
                        let formatted = if far_m <= 0.0 || t >= 1.0 {
                            let dof_near = near_m.max(0.0);
                            format!("inf ({:.2} m - inf)", dof_near)
                        } else {
                            let dof = far_m - near_m;
                            // Use 3 decimal places for small DOF (< 0.02m), otherwise 2
                            let fmt = if dof > 0.0 && dof < 0.02 {
                                format!("{:.3} m ({:.3} - {:.3} m)", dof, near_m, far_m)
                            } else {
                                format!("{:.2} m ({:.2} - {:.2} m)", dof, near_m, far_m)
                            };
                            fmt
                        };
                        output.insert("DOF".to_string(), Value::String(formatted));
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
        "JPEGInterchangeFormat", // IFD0/IFD1 thumbnail offset
        "JPEGInterchangeFormatLength", // IFD0/IFD1 thumbnail length
                                 // Note: NewSubfileType is renamed to SubfileType and decoded
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

    // FocusDistance precision is manufacturer-specific:
    // - Nikon: %.2f (2 decimal places)
    // - Olympus: raw value with trailing zeros trimmed (3 decimal places max)
    // - Minolta: trailing zeros trimmed (e.g., "0.9 m" not "0.90 m")
    // - Others: %.2f (default)
    // The makernote stores high precision; we reformat here after computed fields use it.
    if let Some(Value::String(s)) = output.get("FocusDistance") {
        if let Some(value) = s.strip_suffix(" m").and_then(|v| v.parse::<f64>().ok()) {
            let is_olympus = make_ref.is_some_and(|m| {
                m.contains("OLYMPUS") || m.contains("OM Digital") || m.contains("OM System")
            });
            let is_minolta = make_ref.is_some_and(|m| {
                m.to_uppercase().contains("MINOLTA") || m.to_uppercase().contains("KONICA")
            });
            if is_olympus {
                // Olympus: use 3 decimal places with trailing zeros trimmed
                let formatted = format!("{:.3}", value);
                let formatted = formatted.trim_end_matches('0').trim_end_matches('.');
                output.insert(
                    "FocusDistance".to_string(),
                    Value::String(format!("{} m", formatted)),
                );
            } else if is_minolta {
                // Minolta: use 2 decimal places with trailing zeros trimmed
                let formatted = format!("{:.2}", value);
                let formatted = formatted.trim_end_matches('0').trim_end_matches('.');
                output.insert(
                    "FocusDistance".to_string(),
                    Value::String(format!("{} m", formatted)),
                );
            } else {
                // Nikon and others: use 2 decimal places
                output.insert(
                    "FocusDistance".to_string(),
                    Value::String(format!("{:.2} m", value)),
                );
            }
        }
    }

    // Convert uppercase string values to title case for specific Nikon tags
    // ExifTool normalizes these strings to title case (e.g., "LOW" -> "Low")
    let title_case_tags = ["ToneComp", "AuxiliaryLens", "ImageAdjustment"];
    for tag in title_case_tags {
        if let Some(Value::String(s)) = output.get(tag) {
            // Only convert if all alphabetic chars are uppercase
            if s.chars()
                .filter(|c| c.is_alphabetic())
                .all(|c| c.is_uppercase())
            {
                output.insert(tag.to_string(), Value::String(to_title_case(s)));
            }
        }
    }

    // Remove GPS tags with invalid/placeholder values
    // ExifTool doesn't output GPS tags that have invalid reference values or all-zero coords
    let invalid_gps_tags: Vec<String> = output
        .iter()
        .filter(|(k, v)| {
            k.starts_with("GPS") && {
                if let Value::String(s) = v {
                    s.starts_with("Unknown") || s == "n/a" || s.starts_with("0 deg 0' 0")
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
