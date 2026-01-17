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
    /// Raw value for exiv2 format output (numeric value before decoding)
    pub raw_value: Option<ExifValue>,
    /// exiv2 group name (e.g., "CanonCs", "CanonSi", "NikonPc")
    pub exiv2_group: Option<&'static str>,
    /// exiv2 tag name within the group (e.g., "Macro", "Selftimer")
    pub exiv2_name: Option<&'static str>,
}

impl MakerNoteTag {
    /// Create a new MakerNoteTag with optional exiv2 fields set to None
    pub fn new(tag_id: u16, tag_name: Option<&'static str>, value: ExifValue) -> Self {
        Self {
            tag_id,
            tag_name,
            value,
            raw_value: None,
            exiv2_group: None,
            exiv2_name: None,
        }
    }

    /// Create a MakerNoteTag with exiv2 output support
    pub fn with_exiv2(
        tag_id: u16,
        tag_name: Option<&'static str>,
        value: ExifValue,
        raw_value: ExifValue,
        exiv2_group: &'static str,
        exiv2_name: &'static str,
    ) -> Self {
        Self {
            tag_id,
            tag_name,
            value,
            raw_value: Some(raw_value),
            exiv2_group: Some(exiv2_group),
            exiv2_name: Some(exiv2_name),
        }
    }
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
    parse_maker_notes_with_tiff_data(data, make, None, endian, None, 0, None)
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
/// * `makernote_file_offset` - Optional file offset where MakerNote data starts (for Olympus PreviewImageStart)
pub fn parse_maker_notes_with_tiff_data(
    data: &[u8],
    make: Option<&str>,
    model: Option<&str>,
    endian: Endianness,
    tiff_data: Option<&[u8]>,
    tiff_offset: usize,
    makernote_file_offset: Option<usize>,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let make_lower = make.map(|s| s.to_lowercase());
    let make_str = make_lower.as_deref().unwrap_or("");

    if make_str.contains("canon") {
        canon::parse_canon_maker_notes(data, endian, tiff_data, tiff_offset, model)
    } else if make_str.contains("nikon") {
        nikon::parse_nikon_maker_notes(
            data,
            endian,
            model,
            tiff_data,
            tiff_offset,
            makernote_file_offset,
        )
    } else if make_str.contains("sony") {
        sony::parse_sony_maker_notes(data, endian, tiff_data, tiff_offset, model)
    } else if make_str.contains("fuji") {
        fuji::parse_fuji_maker_notes(data, endian)
    } else if make_str.contains("olympus") {
        olympus::parse_olympus_maker_notes(
            data,
            endian,
            tiff_data,
            tiff_offset,
            makernote_file_offset,
        )
    } else if make_str.contains("panasonic") {
        panasonic::parse_panasonic_maker_notes(data, endian, model, tiff_data, tiff_offset)
    } else if make_str.contains("pentax")
        || make_str.contains("ricoh")
        || make_str.contains("samsung")
    {
        // Samsung cameras (GX-1S, GX-1L, GX10, GX20) use Pentax MakerNote format
        pentax::parse_pentax_maker_notes(data, endian, tiff_data, tiff_offset)
    } else if make_str.contains("minolta") || make_str.contains("konica") {
        minolta::parse_minolta_maker_notes(data, endian, tiff_data, tiff_offset)
    } else if make_str.contains("kodak") || make_str.contains("eastman") {
        kodak::parse_kodak_maker_notes(data, endian)
    } else {
        // Unknown maker, return empty HashMap
        Ok(HashMap::new())
    }
}
