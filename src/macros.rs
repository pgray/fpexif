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
}
