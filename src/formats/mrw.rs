// MRW (Minolta RAW) format support
//
// MRW is Minolta's proprietary RAW format.
// File structure:
// - 0x00 0x4D 0x52 0x4D (MRM\0 - signature, 4 bytes)
// - Data offset (4 bytes, big-endian) - points to raw sensor data
// - Blocks with 4-byte identifiers at offset 8:
//   - PRD (parameter data): camera settings, sensor info
//   - WBG: white balance
//   - RIF: raw image format info
//   - TTW: TIFF/EXIF metadata (contains standard EXIF tags)

use crate::errors::{ExifError, ExifResult};
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Read, Seek, SeekFrom};

const MRW_SIGNATURE: [u8; 4] = [0x00, 0x4D, 0x52, 0x4D]; // "\0MRM"

pub fn extract_exif_segment<R: Read + Seek>(mut reader: R) -> ExifResult<Vec<u8>> {
    // Verify MRW signature
    let mut signature = [0u8; 4];
    reader.read_exact(&mut signature)?;

    if signature != MRW_SIGNATURE {
        return Err(ExifError::Format("Invalid MRW signature".to_string()));
    }

    // Read data offset - this points to raw sensor data, not the metadata blocks
    // Blocks are located between offset 8 and data_offset
    let data_offset = reader.read_u32::<BigEndian>()?;

    // Current position is 8 (after signature + data_offset)
    // Blocks start here and continue until data_offset

    // Look for TTW (TIFF thumbnail/EXIF) block which contains the EXIF data
    loop {
        // Check if we've passed the metadata section
        let current_pos = reader.stream_position()?;
        if current_pos >= data_offset as u64 {
            return Err(ExifError::Format(
                "No TTW block found in MRW file".to_string(),
            ));
        }

        // Read block identifier (4 bytes)
        let mut block_id = [0u8; 4];
        if reader.read_exact(&mut block_id).is_err() {
            return Err(ExifError::Format(
                "No EXIF data found in MRW file".to_string(),
            ));
        }

        // Read block size (4 bytes, big-endian)
        let block_size = match reader.read_u32::<BigEndian>() {
            Ok(size) => size,
            Err(_) => return Err(ExifError::Format("Invalid MRW block structure".to_string())),
        };

        // Check if this is the TTW (TIFF/EXIF) block
        if &block_id == b"\0TTW" {
            // TTW block contains TIFF-structured EXIF data
            let mut ttw_data = vec![0u8; block_size as usize];
            reader.read_exact(&mut ttw_data)?;

            // Check if it starts with TIFF header
            if ttw_data.len() >= 8
                && ((ttw_data[0] == b'I' && ttw_data[1] == b'I')
                    || (ttw_data[0] == b'M' && ttw_data[1] == b'M'))
            {
                // Has TIFF header, wrap in APP1 format
                let mut app1_data = Vec::with_capacity(6 + ttw_data.len());
                app1_data.extend_from_slice(b"Exif\0\0");
                app1_data.extend_from_slice(&ttw_data);
                return Ok(app1_data);
            } else {
                return Err(ExifError::Format(
                    "TTW block has unexpected format".to_string(),
                ));
            }
        }

        // Skip this block and continue to next
        reader.seek(SeekFrom::Current(block_size as i64))?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_invalid_mrw_signature() {
        let data = vec![0x00, 0x00, 0x00, 0x00]; // Invalid signature

        let cursor = Cursor::new(data);
        let result = extract_exif_segment(cursor);

        assert!(result.is_err(), "Should reject invalid MRW signature");
    }

    #[test]
    fn test_valid_mrw_signature() {
        let mut data = Vec::new();
        data.extend_from_slice(&MRW_SIGNATURE); // Valid signature
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x08]); // Data offset (8) - blocks immediately follow

        let cursor = Cursor::new(data);
        let result = extract_exif_segment(cursor);

        // Will fail because we've reached data_offset without finding TTW block
        assert!(result.is_err());
        match result {
            Err(ExifError::Format(msg)) => {
                assert!(msg.contains("No TTW block found"));
            }
            _ => panic!("Expected Format error"),
        }
    }
}
