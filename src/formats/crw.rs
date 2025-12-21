// CRW (Canon Raw v1) format support
//
// CRW files use the CIFF (Camera Image File Format) structure.
// Signature: "HEAPCCDR" at offset 6
//
// Unlike TIFF-based RAW formats, CRW files store metadata in their own
// CIFF structure rather than as standard EXIF data. We parse the CIFF
// structure and build a synthetic EXIF segment.

use crate::errors::ExifResult;
use crate::formats::ciff::CiffParser;
use std::io::{Read, Seek};

pub fn extract_exif_segment<R: Read + Seek>(reader: R) -> ExifResult<Vec<u8>> {
    let mut parser = CiffParser::new(reader)?;

    // Build a synthetic EXIF segment from CIFF metadata
    parser.build_exif_segment()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_crw_signature_validation() {
        // Invalid CRW file (bad signature)
        let mut data = Vec::new();
        data.extend_from_slice(b"II");
        data.extend_from_slice(&26u32.to_le_bytes());
        data.extend_from_slice(b"INVALID!");

        let cursor = Cursor::new(data);
        let result = extract_exif_segment(cursor);

        assert!(result.is_err(), "Should reject invalid CRW signature");
    }
}
