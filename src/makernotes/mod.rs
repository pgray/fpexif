// makernotes/mod.rs - Camera manufacturer-specific maker notes parsing

pub mod canon;
pub mod fuji;
pub mod nikon;
pub mod sony;

use crate::data_types::{Endianness, ExifValue};
use crate::errors::ExifError;
use std::collections::HashMap;

/// Represents a parsed maker note entry
#[derive(Debug, Clone)]
pub struct MakerNoteTag {
    pub tag_id: u16,
    pub tag_name: Option<&'static str>,
    pub value: ExifValue,
}

/// Parse maker notes based on camera make
pub fn parse_maker_notes(
    data: &[u8],
    make: Option<&str>,
    endian: Endianness,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let make_lower = make.map(|s| s.to_lowercase());
    let make_str = make_lower.as_deref().unwrap_or("");

    if make_str.contains("canon") {
        canon::parse_canon_maker_notes(data, endian)
    } else if make_str.contains("nikon") {
        nikon::parse_nikon_maker_notes(data, endian)
    } else if make_str.contains("sony") {
        sony::parse_sony_maker_notes(data, endian)
    } else if make_str.contains("fuji") {
        fuji::parse_fuji_maker_notes(data, endian)
    } else {
        // Unknown maker, return empty HashMap
        Ok(HashMap::new())
    }
}
