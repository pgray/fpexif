// makernotes/mod.rs - Camera manufacturer-specific maker notes parsing

pub mod canon;
pub mod fuji;
pub mod kodak;
pub mod minolta;
pub mod nikon;
pub mod olympus;
pub mod panasonic;
pub mod pentax;
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
///
/// # Arguments
/// * `data` - The maker note data (contents of MakerNote tag)
/// * `make` - Camera make string
/// * `endian` - Byte order
pub fn parse_maker_notes(
    data: &[u8],
    make: Option<&str>,
    endian: Endianness,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    parse_maker_notes_with_tiff_data(data, make, None, endian, None, 0)
}

/// Parse maker notes with access to the full TIFF data
///
/// Some manufacturers (like Canon) use offsets relative to the TIFF header,
/// so we need access to the full TIFF data to resolve those offsets.
///
/// # Arguments
/// * `data` - The maker note data (contents of MakerNote tag)
/// * `make` - Camera make string
/// * `model` - Camera model string (used for model-specific formatting)
/// * `endian` - Byte order
/// * `tiff_data` - Optional full TIFF/EXIF data for resolving absolute offsets
/// * `tiff_offset` - Offset of TIFF header within the full data
pub fn parse_maker_notes_with_tiff_data(
    data: &[u8],
    make: Option<&str>,
    model: Option<&str>,
    endian: Endianness,
    tiff_data: Option<&[u8]>,
    tiff_offset: usize,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let make_lower = make.map(|s| s.to_lowercase());
    let make_str = make_lower.as_deref().unwrap_or("");

    if make_str.contains("canon") {
        canon::parse_canon_maker_notes(data, endian, tiff_data, tiff_offset, model)
    } else if make_str.contains("nikon") {
        nikon::parse_nikon_maker_notes(data, endian)
    } else if make_str.contains("sony") {
        sony::parse_sony_maker_notes(data, endian, tiff_data, tiff_offset)
    } else if make_str.contains("fuji") {
        fuji::parse_fuji_maker_notes(data, endian)
    } else if make_str.contains("olympus") {
        olympus::parse_olympus_maker_notes(data, endian, tiff_data, tiff_offset)
    } else if make_str.contains("panasonic") {
        panasonic::parse_panasonic_maker_notes(data, endian)
    } else if make_str.contains("pentax") || make_str.contains("ricoh") {
        pentax::parse_pentax_maker_notes(data, endian)
    } else if make_str.contains("minolta") || make_str.contains("konica") {
        minolta::parse_minolta_maker_notes(data, endian)
    } else if make_str.contains("kodak") || make_str.contains("eastman") {
        kodak::parse_kodak_maker_notes(data, endian)
    } else {
        // Unknown maker, return empty HashMap
        Ok(HashMap::new())
    }
}
