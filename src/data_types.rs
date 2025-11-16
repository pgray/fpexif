// data_types.rs - EXIF data type definitions
use std::fmt;

#[cfg(feature = "serde")]
use serde::Serialize;

/// Represents the byte order (endianness) of the EXIF data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Endianness {
    /// Little endian (Intel)
    Little,
    /// Big endian (Motorola)
    Big,
}

/// Represents the different EXIF data types
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum ExifValue {
    /// Unsigned byte (8 bits)
    Byte(Vec<u8>),
    /// ASCII string (7 bits, null-terminated)
    Ascii(String),
    /// Unsigned short (16 bits)
    Short(Vec<u16>),
    /// Unsigned long (32 bits)
    Long(Vec<u32>),
    /// Unsigned rational (two 32-bit values: numerator/denominator)
    Rational(Vec<(u32, u32)>),
    /// Signed byte (8 bits)
    SByte(Vec<i8>),
    /// Undefined (8-bit byte that can take any value)
    Undefined(Vec<u8>),
    /// Signed short (16 bits)
    SShort(Vec<i16>),
    /// Signed long (32 bits)
    SLong(Vec<i32>),
    /// Signed rational (two 32-bit values: numerator/denominator)
    SRational(Vec<(i32, i32)>),
    /// IEEE floating point (32 bits)
    Float(Vec<f32>),
    /// IEEE floating point (64 bits)
    Double(Vec<f64>),
}

impl ExifValue {
    /// Get the EXIF type ID for this value
    pub fn type_id(&self) -> u16 {
        match self {
            ExifValue::Byte(_) => 1,
            ExifValue::Ascii(_) => 2,
            ExifValue::Short(_) => 3,
            ExifValue::Long(_) => 4,
            ExifValue::Rational(_) => 5,
            ExifValue::SByte(_) => 6,
            ExifValue::Undefined(_) => 7,
            ExifValue::SShort(_) => 8,
            ExifValue::SLong(_) => 9,
            ExifValue::SRational(_) => 10,
            ExifValue::Float(_) => 11,
            ExifValue::Double(_) => 12,
        }
    }

    /// Get the number of components in this value
    pub fn component_count(&self) -> usize {
        match self {
            ExifValue::Byte(v) => v.len(),
            ExifValue::Ascii(s) => s.len() + 1, // Include null terminator
            ExifValue::Short(v) => v.len(),
            ExifValue::Long(v) => v.len(),
            ExifValue::Rational(v) => v.len(),
            ExifValue::SByte(v) => v.len(),
            ExifValue::Undefined(v) => v.len(),
            ExifValue::SShort(v) => v.len(),
            ExifValue::SLong(v) => v.len(),
            ExifValue::SRational(v) => v.len(),
            ExifValue::Float(v) => v.len(),
            ExifValue::Double(v) => v.len(),
        }
    }
}

impl fmt::Display for ExifValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExifValue::Byte(v) => write!(f, "{:?}", v),
            ExifValue::Ascii(s) => write!(f, "{}", s),
            ExifValue::Short(v) => write!(f, "{:?}", v),
            ExifValue::Long(v) => write!(f, "{:?}", v),
            ExifValue::Rational(v) => {
                if v.len() == 1 {
                    let (num, den) = v[0];
                    if den == 0 {
                        write!(f, "∞")
                    } else {
                        write!(f, "{}/{} ({})", num, den, num as f64 / den as f64)
                    }
                } else {
                    write!(f, "{:?}", v)
                }
            }
            ExifValue::SByte(v) => write!(f, "{:?}", v),
            ExifValue::Undefined(v) => write!(f, "0x{}", hex::encode(v)),
            ExifValue::SShort(v) => write!(f, "{:?}", v),
            ExifValue::SLong(v) => write!(f, "{:?}", v),
            ExifValue::SRational(v) => {
                if v.len() == 1 {
                    let (num, den) = v[0];
                    if den == 0 {
                        if num < 0 {
                            write!(f, "-∞")
                        } else {
                            write!(f, "∞")
                        }
                    } else {
                        write!(f, "{}/{} ({})", num, den, num as f64 / den as f64)
                    }
                } else {
                    write!(f, "{:?}", v)
                }
            }
            ExifValue::Float(v) => write!(f, "{:?}", v),
            ExifValue::Double(v) => write!(f, "{:?}", v),
        }
    }
}
