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
