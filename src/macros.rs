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

/// Safely read little-endian bytes with proper bounds checking
///
/// This macro provides a consistent, safe way to read multi-byte values from byte arrays.
/// It handles bounds checking and type conversion in a single call.
///
/// # Examples
///
/// ```ignore
/// // Read a u16 from bytes 2-4
/// if let Some(val) = read_le_bytes!(data, 2, u16) {
///     println!("Value: {}", val);
/// }
///
/// // Read a u32 from bytes 4-8
/// let timestamp = read_le_bytes!(data, 4, u32).unwrap_or(0);
/// ```
#[macro_export]
macro_rules! read_le_bytes {
    ($data:expr, $offset:expr, $type:ty) => {
        $data
            .get($offset..$offset + std::mem::size_of::<$type>())
            .and_then(|slice| slice.try_into().ok())
            .map(<$type>::from_le_bytes)
    };
}

/// Format 4 values from an array with a custom separator
///
/// This macro standardizes the formatting of 4-value arrays commonly used for
/// RGBG white balance values, coordinate data, and similar patterns.
///
/// # Examples
///
/// ```ignore
/// // Format RGBG values with space separator
/// let wb_string = format_4_values!(rggb_data, 0, " ");
/// // Output: "128 130 125 127"
///
/// // Format with comma separator
/// let coord_string = format_4_values!(coord_data, 4, ",");
/// // Output: "100,200,150,175"
/// ```
#[macro_export]
macro_rules! format_4_values {
    ($data:expr, $start_idx:expr, $separator:expr) => {
        format!(
            "{}{}{}{}{}{}{}",
            $data.get($start_idx).unwrap_or(&0),
            $separator,
            $data.get($start_idx + 1).unwrap_or(&0),
            $separator,
            $data.get($start_idx + 2).unwrap_or(&0),
            $separator,
            $data.get($start_idx + 3).unwrap_or(&0)
        )
    };
}

/// Decode values with special handling for markers and signed conversion
///
/// This macro handles the common pattern in Canon EXIF data where certain values
/// (like 0x7fff) indicate "n/a" or special states, and values need to be converted
/// from unsigned to signed with proper formatting.
///
/// # Examples
///
/// ```ignore
/// // Canon contrast/saturation/sharpness pattern
/// let result = decode_with_special_values!(value, 0x7fff, "n/a", "Normal");
/// // Returns: "n/a" for 0x7fff, "Normal" for 0, "+2" for 2, "-1" for signed -1
/// ```
#[macro_export]
macro_rules! decode_with_special_values {
    ($value:expr, $special_val:expr, $special_str:expr, $zero_str:expr) => {
        if $value == $special_val {
            $special_str.to_string()
        } else {
            let signed_val = if $value > 0xfff0 {
                ($value as i32) - 0x10000
            } else {
                $value as i32
            };
            if signed_val == 0 {
                $zero_str.to_string()
            } else if signed_val > 0 {
                format!("+{}", signed_val)
            } else {
                signed_val.to_string()
            }
        }
    };
}

/// Extract strings with null termination and space padding cleanup
///
/// This macro standardizes the extraction of firmware versions, model names,
/// and other string data that may contain null terminators or space padding.
///
/// # Examples
///
/// ```ignore
/// // Extract firmware version (20 bytes starting at offset 2)
/// let firmware = extract_string!(data, 2, 20);
/// // Output: "1.2.3" (instead of "1.2.3\0\0\0   ")
///
/// // Extract model name (32 bytes starting at offset 40)
/// let model = extract_string!(data, 40, 32);
/// ```
#[macro_export]
macro_rules! extract_string {
    ($data:expr, $start:expr, $len:expr) => {
        String::from_utf8_lossy(&$data[$start..$start + $len])
            .trim_end_matches('\0')
            .trim_end()
            .to_string()
    };
}

/// Decode bitmask values into human-readable flag lists
///
/// This macro handles the common pattern of decoding bitwise flags into
/// comma-separated human-readable strings, commonly used for shooting modes,
/// AF modes, and other multi-state settings.
///
/// # Examples
///
/// ```ignore
/// // Nikon shooting mode pattern
/// let mode_str = decode_bitmask!(value, {
///     0x1 => "AF-S",
///     0x2 => "AF-C",
///     0x4 => "AF-A",
///     0x8 => "Manual",
/// });
/// // Output: "AF-S, AF-C" for value 0x3
/// ```
#[macro_export]
macro_rules! decode_bitmask {
    ($value:expr, { $($flag:expr => $str:expr),* $(,)? }) => {
        {
            let mut parts = Vec::new();
            $(
                if $value & $flag != 0 {
                    parts.push($str);
                }
            )*
            parts.join(", ")
        }
    };
}

/// Create simple wrapper functions for function delegation
///
/// This macro reduces boilerplate when creating thin wrapper functions
/// that simply call another function with the same signature.
///
/// # Examples
///
/// ```ignore
/// // Instead of manually writing:
/// // pub fn decode_focus_mode_exiftool(value: u16) -> &'static str {
/// //     decode_pentax_focus_mode_exiftool(value)
/// // }
///
/// // Use the macro:
/// define_wrapper!(decode_focus_mode_exiftool, decode_pentax_focus_mode_exiftool, u16);
/// ```
#[macro_export]
macro_rules! define_wrapper {
    ($wrapper_name:ident, $inner_fn:ident, $type:ty) => {
        pub fn $wrapper_name(value: $type) -> &'static str {
            $inner_fn(value)
        }
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

    // Tests for new macros

    #[test]
    fn test_read_le_bytes() {
        let data = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];

        // Test u16 reading
        let val = read_le_bytes!(&data, 0, u16).unwrap();
        assert_eq!(val, 0x3412);

        // Test u32 reading
        let val = read_le_bytes!(&data, 0, u32).unwrap();
        assert_eq!(val, 0x78563412);

        // Test i32 reading (negative value)
        let data_signed = [0xFF, 0xFF, 0xFF, 0xFF];
        let val = read_le_bytes!(&data_signed, 0, i32).unwrap();
        assert_eq!(val, -1);

        // Test out of bounds reading
        let val = read_le_bytes!(&data, 6, u16); // Only 2 bytes available at end
        assert!(val.is_some());
        let val = read_le_bytes!(&data, 7, u16); // Only 1 byte available
        assert!(val.is_none());
    }

    #[test]
    fn test_format_4_values() {
        let data = [10, 20, 30, 40, 50, 60, 70, 80];

        // Test with space separator
        let result = format_4_values!(&data, 0, " ");
        assert_eq!(result, "10 20 30 40");

        // Test with comma separator
        let result = format_4_values!(&data, 2, ",");
        assert_eq!(result, "30,40,50,60");

        // Test with index near end (handles missing values gracefully)
        let result = format_4_values!(&data, 6, " ");
        assert_eq!(result, "70 80 0 0");
    }

    #[test]
    fn test_decode_with_special_values() {
        // Test special value (0x7fff)
        let result = decode_with_special_values!(0x7fff, 0x7fff, "n/a", "Normal");
        assert_eq!(result, "n/a");

        // Test zero value
        let result = decode_with_special_values!(0, 0x7fff, "n/a", "Normal");
        assert_eq!(result, "Normal");

        // Test positive values (with + prefix)
        let result = decode_with_special_values!(2, 0x7fff, "n/a", "Normal");
        assert_eq!(result, "+2");

        // Test negative values (signed conversion > 0xfff0)
        let result = decode_with_special_values!(0xffff, 0x7fff, "n/a", "Normal");
        assert_eq!(result, "-1");

        let result = decode_with_special_values!(0xfffe, 0x7fff, "n/a", "Normal");
        assert_eq!(result, "-2");
    }

    #[test]
    fn test_extract_string() {
        // Test normal string
        let data = b"Hello World\0\0\0\0";
        let result = extract_string!(data, 0, 15);
        assert_eq!(result, "Hello World");

        // Test string with space padding
        let data = b"Firmware 1.2   ";
        let result = extract_string!(data, 0, 15);
        assert_eq!(result, "Firmware 1.2");

        // Test string with embedded nulls (trim_end only removes trailing nulls)
        let data = b"Model\0XYZ\0\0\0\0";
        let result = extract_string!(data, 0, 12);
        assert_eq!(result, "Model\0XYZ");
    }

    #[test]
    fn test_decode_bitmask() {
        // Test single flag
        let result = decode_bitmask!(0x1, {
            0x1 => "Flag1",
            0x2 => "Flag2",
            0x4 => "Flag3",
        });
        assert_eq!(result, "Flag1");

        // Test multiple flags
        let result = decode_bitmask!(0x5, {
            0x1 => "AF-S",
            0x2 => "AF-C",
            0x4 => "Manual",
        });
        assert_eq!(result, "AF-S, Manual");

        // Test no flags
        let result = decode_bitmask!(0x0, {
            0x1 => "Flag1",
            0x2 => "Flag2",
        });
        assert_eq!(result, "");

        // Test all flags
        let result = decode_bitmask!(0x7, {
            0x1 => "Flag1",
            0x2 => "Flag2",
            0x4 => "Flag3",
        });
        assert_eq!(result, "Flag1, Flag2, Flag3");
    }

    #[test]
    fn test_define_wrapper() {
        // Since we can't test the macro directly (it generates functions),
        // we'll test the pattern it replaces
        fn inner_test_fn(value: u16) -> &'static str {
            match value {
                1 => "One",
                2 => "Two",
                _ => "Other",
            }
        }

        // This is what the macro would generate:
        fn wrapper_test_fn(value: u16) -> &'static str {
            inner_test_fn(value)
        }

        assert_eq!(wrapper_test_fn(1), "One");
        assert_eq!(wrapper_test_fn(2), "Two");
        assert_eq!(wrapper_test_fn(3), "Other");
    }
}
