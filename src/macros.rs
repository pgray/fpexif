// macros.rs - Macros for simplifying tag decoder definitions

/// Define tag decoder functions for both ExifTool and exiv2 formats
///
/// This macro generates two decoder functions: one with `_exiftool` suffix and one with `_exiv2` suffix.
///
/// # Examples
///
/// ## Different mappings for exiftool and exiv2:
/// ```ignore
/// define_tag_decoder! {
///     white_balance,
///     exiftool: {
///         1 => "Auto",
///         2 => "Daylight",
///         3 => "Cloudy",
///     },
///     exiv2: {
///         1 => "Auto",
///         2 => "Daylight",
///         3 => "Cloudy",
///     }
/// }
/// ```
///
/// ## Same mapping for both formats:
/// ```ignore
/// define_tag_decoder! {
///     focus_mode,
///     both: {
///         1 => "Auto",
///         2 => "Manual",
///         4 => "Auto, Focus button",
///     }
/// }
/// ```
///
/// ## With explicit type (u32, i16, i32, u8, etc.):
/// ```ignore
/// define_tag_decoder! {
///     vignetting_correction,
///     type: u32,
///     both: {
///         0 => "Off",
///         2 => "Auto",
///         0xFFFFFFFF => "N/A",
///     }
/// }
/// ```
///
/// ## Different mappings with explicit type:
/// ```ignore
/// define_tag_decoder! {
///     adjustment,
///     type: i32,
///     exiftool: {
///         -2 => "-2",
///         -1 => "-1",
///         0 => "0",
///         1 => "+1",
///         2 => "+2",
///     },
///     exiv2: {
///         -2 => "Minus 2",
///         -1 => "Minus 1",
///         0 => "Normal",
///         1 => "Plus 1",
///         2 => "Plus 2",
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_tag_decoder {
    // Pattern for different mappings (exiftool and exiv2)
    (
        $name:ident,
        exiftool: {
            $($et_value:expr => $et_str:expr),* $(,)?
        },
        exiv2: {
            $($ev_value:expr => $ev_str:expr),* $(,)?
        }
    ) => {
        ::paste::paste! {
            #[doc = "Decode " $name " value - ExifTool format"]
            pub fn [<decode_ $name _exiftool>](value: u16) -> &'static str {
                match value {
                    $($et_value => $et_str,)*
                    _ => "Unknown",
                }
            }

            #[doc = "Decode " $name " value - exiv2 format"]
            pub fn [<decode_ $name _exiv2>](value: u16) -> &'static str {
                match value {
                    $($ev_value => $ev_str,)*
                    _ => "Unknown",
                }
            }
        }
    };

    // Pattern for same mapping (both formats identical)
    (
        $name:ident,
        both: {
            $($value:expr => $str:expr),* $(,)?
        }
    ) => {
        ::paste::paste! {
            #[doc = "Decode " $name " value - ExifTool format"]
            pub fn [<decode_ $name _exiftool>](value: u16) -> &'static str {
                match value {
                    $($value => $str,)*
                    _ => "Unknown",
                }
            }

            #[doc = "Decode " $name " value - exiv2 format"]
            pub fn [<decode_ $name _exiv2>](value: u16) -> &'static str {
                match value {
                    $($value => $str,)*
                    _ => "Unknown",
                }
            }
        }
    };

    // Pattern for different mappings with explicit type
    (
        $name:ident,
        type: $t:ty,
        exiftool: {
            $($et_value:expr => $et_str:expr),* $(,)?
        },
        exiv2: {
            $($ev_value:expr => $ev_str:expr),* $(,)?
        }
    ) => {
        ::paste::paste! {
            #[doc = "Decode " $name " value - ExifTool format"]
            pub fn [<decode_ $name _exiftool>](value: $t) -> &'static str {
                match value {
                    $($et_value => $et_str,)*
                    _ => "Unknown",
                }
            }

            #[doc = "Decode " $name " value - exiv2 format"]
            pub fn [<decode_ $name _exiv2>](value: $t) -> &'static str {
                match value {
                    $($ev_value => $ev_str,)*
                    _ => "Unknown",
                }
            }
        }
    };

    // Pattern for same mapping with explicit type (both formats identical)
    (
        $name:ident,
        type: $t:ty,
        both: {
            $($value:expr => $str:expr),* $(,)?
        }
    ) => {
        ::paste::paste! {
            #[doc = "Decode " $name " value - ExifTool format"]
            pub fn [<decode_ $name _exiftool>](value: $t) -> &'static str {
                match value {
                    $($value => $str,)*
                    _ => "Unknown",
                }
            }

            #[doc = "Decode " $name " value - exiv2 format"]
            pub fn [<decode_ $name _exiv2>](value: $t) -> &'static str {
                match value {
                    $($value => $str,)*
                    _ => "Unknown",
                }
            }
        }
    };
}

/// Decode a field from a binary array using an enum decoder
///
/// This macro reduces boilerplate for extracting and decoding enum fields from binary arrays.
///
/// # Examples
///
/// ```ignore
/// use std::collections::HashMap;
/// use crate::ExifValue;
///
/// fn decode_data(data: &[u16]) -> HashMap<String, ExifValue> {
///     let mut decoded = HashMap::new();
///
///     // Simple decode at index 1
///     decode_field!(decoded, data, 1, "MacroMode", decode_macro_mode_exiftool);
///
///     // With skip condition (skip if value == 0)
///     decode_field!(decoded, data, 19, "AFPoint", decode_af_point_exiftool, skip_if: 0);
///
///     // With skip condition (skip if value == 0x7fff)
///     decode_field!(decoded, data, 13, "Contrast", decode_contrast_exiftool, skip_if: 0x7fff);
///
///     // Raw numeric output (no decoder function)
///     decode_field!(decoded, data, 5, "RawValue", raw_u16);
///
///     // With i16 cast before decoding
///     decode_field!(decoded, data, 9, "RecordMode", decode_record_mode_exiftool, cast: i16);
///
///     decoded
/// }
/// ```
#[macro_export]
macro_rules! decode_field {
    // Raw u16 output - just insert the numeric value (MUST be before generic $decoder:expr patterns)
    ($map:expr, $data:expr, $index:expr, $name:expr, raw_u16) => {
        if $data.len() > $index {
            $map.insert($name.to_string(), ExifValue::Short(vec![$data[$index]]));
        }
    };

    // Raw u16 with skip condition
    ($map:expr, $data:expr, $index:expr, $name:expr, raw_u16, skip_if: $skip_val:expr) => {
        if $data.len() > $index && $data[$index] != $skip_val {
            $map.insert($name.to_string(), ExifValue::Short(vec![$data[$index]]));
        }
    };

    // Raw i16 (signed) output
    ($map:expr, $data:expr, $index:expr, $name:expr, raw_i16) => {
        if $data.len() > $index {
            $map.insert(
                $name.to_string(),
                ExifValue::SShort(vec![$data[$index] as i16]),
            );
        }
    };

    // With skip condition - skip if value equals skip_value
    ($map:expr, $data:expr, $index:expr, $name:expr, $decoder:expr, skip_if: $skip_val:expr) => {
        if $data.len() > $index && $data[$index] != $skip_val {
            $map.insert(
                $name.to_string(),
                ExifValue::Ascii($decoder($data[$index]).to_string()),
            );
        }
    };

    // With i16 cast before decoding
    ($map:expr, $data:expr, $index:expr, $name:expr, $decoder:expr, cast: i16) => {
        if $data.len() > $index {
            $map.insert(
                $name.to_string(),
                ExifValue::Ascii($decoder($data[$index] as i16).to_string()),
            );
        }
    };

    // With i16 cast and skip condition
    ($map:expr, $data:expr, $index:expr, $name:expr, $decoder:expr, cast: i16, skip_if: $skip_val:expr) => {
        if $data.len() > $index && $data[$index] != $skip_val {
            $map.insert(
                $name.to_string(),
                ExifValue::Ascii($decoder($data[$index] as i16).to_string()),
            );
        }
    };

    // Simple decode - call decoder function on data[index] (MUST be last - catches everything)
    ($map:expr, $data:expr, $index:expr, $name:expr, $decoder:expr) => {
        if $data.len() > $index {
            $map.insert(
                $name.to_string(),
                ExifValue::Ascii($decoder($data[$index]).to_string()),
            );
        }
    };
}

/// Generate PictureStyle fields for Canon ColorData
///
/// Each picture style has 6 fields at fixed offsets:
/// - +0x00: Contrast
/// - +0x04: Sharpness (actually +0x02 in i16 index)
/// - +0x08: Saturation (actually +0x04)
/// - +0x0c: ColorTone (actually +0x06)
/// - +0x10: FilterEffect (actually +0x08) - typically not decoded
/// - +0x14: ToningEffect (actually +0x0a) - typically not decoded
///
/// # Example
/// ```ignore
/// decode_picture_styles!(decoded, data,
///     0x00 => "Standard",
///     0x0c => "Portrait",
///     0x18 => "Landscape",
/// );
/// ```
#[macro_export]
macro_rules! decode_picture_styles {
    ($map:expr, $data:expr, $($base_idx:expr => $style:literal),* $(,)?) => {
        $(
            // Contrast
            if $data.len() > $base_idx {
                let val = $data[$base_idx] as i16;
                $map.insert(
                    concat!("Contrast", $style).to_string(),
                    ExifValue::SShort(vec![val]),
                );
            }
            // Sharpness (+1 index = +2 bytes)
            if $data.len() > $base_idx + 1 {
                let val = $data[$base_idx + 1] as i16;
                $map.insert(
                    concat!("Sharpness", $style).to_string(),
                    ExifValue::SShort(vec![val]),
                );
            }
            // Saturation (+2 index = +4 bytes)
            if $data.len() > $base_idx + 2 {
                let val = $data[$base_idx + 2] as i16;
                $map.insert(
                    concat!("Saturation", $style).to_string(),
                    ExifValue::SShort(vec![val]),
                );
            }
            // ColorTone (+3 index = +6 bytes)
            if $data.len() > $base_idx + 3 {
                let val = $data[$base_idx + 3] as i16;
                $map.insert(
                    concat!("ColorTone", $style).to_string(),
                    ExifValue::SShort(vec![val]),
                );
            }
        )*
    };
}

#[cfg(test)]
mod tests {
    // Test with different mappings for exiftool and exiv2
    define_tag_decoder! {
        test_white_balance,
        exiftool: {
            1 => "Auto",
            2 => "Daylight",
            3 => "Cloudy",
            4 => "Incandescent",
        },
        exiv2: {
            1 => "Auto",
            2 => "Daylight",
            3 => "Cloudy",
            4 => "Halogen",
        }
    }

    // Test with same mapping for both formats
    define_tag_decoder! {
        test_focus_mode,
        both: {
            1 => "Auto",
            2 => "Manual",
            4 => "Auto, Focus button",
            5 => "Auto, Continuous",
        }
    }

    #[test]
    fn test_decoder_different_mappings() {
        // Test exiftool version
        assert_eq!(decode_test_white_balance_exiftool(1), "Auto");
        assert_eq!(decode_test_white_balance_exiftool(2), "Daylight");
        assert_eq!(decode_test_white_balance_exiftool(4), "Incandescent");
        assert_eq!(decode_test_white_balance_exiftool(99), "Unknown");

        // Test exiv2 version
        assert_eq!(decode_test_white_balance_exiv2(1), "Auto");
        assert_eq!(decode_test_white_balance_exiv2(2), "Daylight");
        assert_eq!(decode_test_white_balance_exiv2(4), "Halogen");
        assert_eq!(decode_test_white_balance_exiv2(99), "Unknown");
    }

    #[test]
    fn test_decoder_same_mapping() {
        // Test exiftool version
        assert_eq!(decode_test_focus_mode_exiftool(1), "Auto");
        assert_eq!(decode_test_focus_mode_exiftool(2), "Manual");
        assert_eq!(decode_test_focus_mode_exiftool(5), "Auto, Continuous");
        assert_eq!(decode_test_focus_mode_exiftool(99), "Unknown");

        // Test exiv2 version (should be identical)
        assert_eq!(decode_test_focus_mode_exiv2(1), "Auto");
        assert_eq!(decode_test_focus_mode_exiv2(2), "Manual");
        assert_eq!(decode_test_focus_mode_exiv2(5), "Auto, Continuous");
        assert_eq!(decode_test_focus_mode_exiv2(99), "Unknown");
    }

    // Test with u32 type and same mapping
    define_tag_decoder! {
        test_vignetting_u32,
        type: u32,
        both: {
            0 => "Off",
            2 => "Auto",
            0xFFFFFFFF => "N/A",
        }
    }

    // Test with i32 type and different mappings
    define_tag_decoder! {
        test_adjustment_i32,
        type: i32,
        exiftool: {
            -2 => "-2",
            -1 => "-1",
            0 => "0",
            1 => "+1",
            2 => "+2",
        },
        exiv2: {
            -2 => "Minus 2",
            -1 => "Minus 1",
            0 => "Normal",
            1 => "Plus 1",
            2 => "Plus 2",
        }
    }

    // Test with u8 type
    define_tag_decoder! {
        test_orientation_u8,
        type: u8,
        both: {
            0 => "Normal",
            1 => "Rotated 90 CW",
            2 => "Rotated 180",
            3 => "Rotated 90 CCW",
        }
    }

    #[test]
    fn test_decoder_typed_u32() {
        assert_eq!(decode_test_vignetting_u32_exiftool(0), "Off");
        assert_eq!(decode_test_vignetting_u32_exiftool(2), "Auto");
        assert_eq!(decode_test_vignetting_u32_exiftool(0xFFFFFFFF), "N/A");
        assert_eq!(decode_test_vignetting_u32_exiftool(99), "Unknown");

        assert_eq!(decode_test_vignetting_u32_exiv2(0), "Off");
        assert_eq!(decode_test_vignetting_u32_exiv2(2), "Auto");
    }

    #[test]
    fn test_decoder_typed_i32() {
        assert_eq!(decode_test_adjustment_i32_exiftool(-2), "-2");
        assert_eq!(decode_test_adjustment_i32_exiftool(-1), "-1");
        assert_eq!(decode_test_adjustment_i32_exiftool(0), "0");
        assert_eq!(decode_test_adjustment_i32_exiftool(1), "+1");
        assert_eq!(decode_test_adjustment_i32_exiftool(2), "+2");
        assert_eq!(decode_test_adjustment_i32_exiftool(99), "Unknown");

        assert_eq!(decode_test_adjustment_i32_exiv2(-2), "Minus 2");
        assert_eq!(decode_test_adjustment_i32_exiv2(0), "Normal");
        assert_eq!(decode_test_adjustment_i32_exiv2(1), "Plus 1");
    }

    #[test]
    fn test_decoder_typed_u8() {
        assert_eq!(decode_test_orientation_u8_exiftool(0), "Normal");
        assert_eq!(decode_test_orientation_u8_exiftool(1), "Rotated 90 CW");
        assert_eq!(decode_test_orientation_u8_exiftool(255), "Unknown");

        assert_eq!(decode_test_orientation_u8_exiv2(2), "Rotated 180");
        assert_eq!(decode_test_orientation_u8_exiv2(3), "Rotated 90 CCW");
    }
}
