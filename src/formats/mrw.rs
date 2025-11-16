// MRW (Minolta RAW) format support
//
// MRW is Minolta's proprietary RAW format.
// File structure:
// - 0x00 0x4D 0x52 0x4D (MRM\0 - signature)
// - Data offset (4 bytes, big-endian)
// - Data size (4 bytes, big-endian)
// - Blocks with 4-byte identifiers: PRD (parameter data), TTW (thumbnail), etc.

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

    // Read data offset and size
    let data_offset = reader.read_u32::<BigEndian>()?;
    let _data_size = reader.read_u32::<BigEndian>()?;

    // Seek to data section
    reader.seek(SeekFrom::Start(data_offset as u64))?;

    // Look for PRD (parameter data) block which contains EXIF
    loop {
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

        // Check if this is the PRD (parameter data) block
        if &block_id == b"\0PRD" {
            // PRD block contains EXIF-like data
            // The structure is similar to TIFF IFD
            let mut prd_data = vec![0u8; block_size as usize];
            reader.read_exact(&mut prd_data)?;

            // Check if it starts with TIFF header
            if prd_data.len() >= 8
                && ((prd_data[0] == b'I' && prd_data[1] == b'I')
                    || (prd_data[0] == b'M' && prd_data[1] == b'M'))
            {
                // Has TIFF header, wrap in APP1 format
                let mut app1_data = Vec::with_capacity(6 + prd_data.len());
                app1_data.extend_from_slice(b"Exif\0\0");
                app1_data.extend_from_slice(&prd_data);
                return Ok(app1_data);
            } else {
                return Err(ExifError::Format(
                    "PRD block has unexpected format".to_string(),
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
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x10]); // Data offset (16)
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x20]); // Data size (32)

        let cursor = Cursor::new(data);
        let result = extract_exif_segment(cursor);

        // Will fail because we don't have actual PRD block, but signature should pass
        assert!(result.is_err());
        match result {
            Err(ExifError::Format(msg)) => {
                assert!(msg.contains("No EXIF data found"));
            }
            _ => panic!("Expected Format error"),
        }
    }
}
