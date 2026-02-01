// makernotes/hasselblad.rs - Hasselblad maker notes parsing
//
// Reference files:
// - exiftool/lib/Image/ExifTool/MakerNotes.pm (shows Hasselblad handling)
// - exiftool/html/makernote_types.html (documents Hasselblad formats)
//
// Hasselblad cameras use two different makernote formats:
//
// 1. VHAB Format (Sony-based):
//    - Models: HV, Lunar, Stellar
//    - Format: Uses Sony makernote structure with "VHAB     \0" header
//    - ExifTool: Parsed as Sony::Main (see MakerNotes.pm line 1034)
//    - Implementation: Delegated to Sony parser
//
// 2. Native Hasselblad Format:
//    - Models: X1D, X1D II 50C, CFV II 50C/907X, CFV-50c/500, HB4116
//    - Format: Standard IFD format (little-endian)
//    - ExifTool: Parsed as Unknown::Main (no tag definitions)
//    - Implementation: Stub (returns empty HashMap until test data available)
//
// Note: ExifTool does not have a dedicated Hasselblad.pm module. VHAB models
// are treated as Sony cameras, and native models have no defined tags.

use crate::data_types::Endianness;
use crate::errors::ExifError;
use crate::makernotes::sony;
use crate::makernotes::MakerNoteTag;
use std::collections::HashMap;

/// Parse Hasselblad maker notes
///
/// This function detects the makernote format and delegates accordingly:
/// - VHAB models (HV, Lunar, Stellar) -> Sony parser
/// - Native models (X1D, CFV, etc.) -> Stub (no tag definitions available)
pub fn parse_hasselblad_maker_notes(
    data: &[u8],
    endian: Endianness,
    tiff_data: Option<&[u8]>,
    tiff_offset: usize,
    model: Option<&str>,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    // Check if this is a VHAB-based Hasselblad (uses Sony format)
    if data.len() >= 12 && data.starts_with(b"VHAB     \0") {
        // VHAB models (HV, Lunar, Stellar) use Sony makernote format
        // Delegate to Sony parser which handles the "SONY DSC \0" / "VHAB     \0" headers
        return sony::parse_sony_maker_notes(data, endian, tiff_data, tiff_offset, model);
    }

    // Native Hasselblad format (X1D, CFV, etc.)
    // These use standard IFD format but ExifTool has no tag definitions
    // Return empty HashMap until we have test data and can implement tags
    Ok(HashMap::new())
}
