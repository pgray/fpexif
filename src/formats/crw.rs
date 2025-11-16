// CRW (Canon Raw v1) format support
//
// CRW files use the CIFF (Camera Image File Format) structure.
// Signature: "HEAPCCDR" at offset 6

use crate::errors::{ExifError, ExifResult};
use crate::formats::ciff::CiffParser;
use std::io::{Read, Seek};

pub fn extract_exif_segment<R: Read + Seek>(reader: R) -> ExifResult<Vec<u8>> {
    let mut parser = CiffParser::new(reader)?;

    match parser.find_exif()? {
        Some(exif_data) => {
            // Check if the data already has TIFF header
            if exif_data.len() >= 8
                && ((exif_data[0] == b'I' && exif_data[1] == b'I')
                    || (exif_data[0] == b'M' && exif_data[1] == b'M'))
            {
                // Already has TIFF header, wrap in APP1 format
                let mut app1_data = Vec::with_capacity(6 + exif_data.len());
                app1_data.extend_from_slice(b"Exif\0\0");
                app1_data.extend_from_slice(&exif_data);
                Ok(app1_data)
            } else {
                // Raw EXIF data without TIFF header
                // In some CRW files, EXIF might be stored differently
                Err(ExifError::Format(
                    "EXIF data in CRW file has unexpected format".to_string(),
                ))
            }
        }
        None => Err(ExifError::Format(
            "No EXIF data found in CRW file".to_string(),
        )),
    }
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
