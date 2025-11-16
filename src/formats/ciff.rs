// CIFF (Camera Image File Format) parser for Canon CRW files
//
// CIFF is Canon's proprietary format used in older Canon cameras (before CR2).
// It uses a "heap" structure where data blocks are referenced by offsets.

use crate::errors::{ExifError, ExifResult};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Seek, SeekFrom};

const CIFF_SIGNATURE: &[u8; 8] = b"HEAPCCDR";
const EXIF_TAG_TYPE: u16 = 0x0805; // EXIF information block tag

#[derive(Debug)]
pub struct CiffHeader {
    pub is_little_endian: bool,
    pub header_length: u32,
    pub root_dir_offset: u32,
}

#[derive(Debug)]
pub struct CiffDirectoryEntry {
    pub tag: u16,
    pub size: u32,
    pub offset: u32,
}

pub struct CiffParser<R: Read + Seek> {
    reader: R,
    header: CiffHeader,
}

impl<R: Read + Seek> CiffParser<R> {
    pub fn new(mut reader: R) -> ExifResult<Self> {
        // Read byte order marker
        let mut byte_order = [0u8; 2];
        reader.read_exact(&mut byte_order)?;

        let is_little_endian = match &byte_order {
            b"II" => true,
            b"MM" => false,
            _ => {
                return Err(ExifError::Format(
                    "Invalid CIFF byte order marker".to_string(),
                ))
            }
        };

        // Read header length
        let header_length = reader.read_u32::<LittleEndian>()?;

        // Verify CIFF signature
        let mut signature = [0u8; 8];
        reader.read_exact(&mut signature)?;
        if &signature != CIFF_SIGNATURE {
            return Err(ExifError::Format("Invalid CIFF signature".to_string()));
        }

        // Skip version (2 bytes) and reserved (8 bytes)
        reader.seek(SeekFrom::Current(10))?;

        // Read offset to root directory
        let root_dir_offset = reader.read_u32::<LittleEndian>()?;

        let header = CiffHeader {
            is_little_endian,
            header_length,
            root_dir_offset,
        };

        Ok(CiffParser { reader, header })
    }

    /// Read directory entries at the given offset
    fn read_directory(&mut self, offset: u32) -> ExifResult<Vec<CiffDirectoryEntry>> {
        self.reader.seek(SeekFrom::Start(offset as u64))?;

        // Read number of entries (2 bytes)
        let entry_count = self.reader.read_u16::<LittleEndian>()?;

        // Sanity check: limit to reasonable number of entries to prevent excessive allocation
        if entry_count > 10000 {
            return Err(ExifError::Format(format!(
                "Invalid CIFF directory: too many entries ({})",
                entry_count
            )));
        }

        let mut entries = Vec::with_capacity(entry_count as usize);

        // Each entry is 10 bytes: tag (2) + size (4) + offset (4)
        for _ in 0..entry_count {
            let tag = self.reader.read_u16::<LittleEndian>()?;
            let size = self.reader.read_u32::<LittleEndian>()?;
            let offset = self.reader.read_u32::<LittleEndian>()?;

            entries.push(CiffDirectoryEntry { tag, size, offset });
        }

        Ok(entries)
    }

    /// Find EXIF data in the CIFF structure
    pub fn find_exif(&mut self) -> ExifResult<Option<Vec<u8>>> {
        // Read root directory
        let entries = self.read_directory(self.header.root_dir_offset)?;

        // Look for EXIF tag (0x0805)
        for entry in entries {
            if entry.tag == EXIF_TAG_TYPE {
                // Found EXIF data - read it
                self.reader.seek(SeekFrom::Start(entry.offset as u64))?;
                let mut exif_data = vec![0u8; entry.size as usize];
                self.reader.read_exact(&mut exif_data)?;
                return Ok(Some(exif_data));
            }

            // CIFF directories can be nested - check if this is a subdirectory
            // Directory entries have the high bit set in their tag
            if (entry.tag & 0xC000) == 0xC000 {
                // This is a subdirectory, search it recursively
                if let Ok(subdir_entries) = self.read_directory(entry.offset) {
                    for subentry in subdir_entries {
                        if subentry.tag == EXIF_TAG_TYPE {
                            self.reader.seek(SeekFrom::Start(subentry.offset as u64))?;
                            let mut exif_data = vec![0u8; subentry.size as usize];
                            self.reader.read_exact(&mut exif_data)?;
                            return Ok(Some(exif_data));
                        }
                    }
                }
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_invalid_signature() {
        let mut data = Vec::new();
        data.extend_from_slice(b"II"); // Byte order
        data.extend_from_slice(&26u32.to_le_bytes()); // Header length
        data.extend_from_slice(b"INVALID!"); // Bad signature

        let cursor = Cursor::new(data);
        let result = CiffParser::new(cursor);

        assert!(result.is_err());
    }

    #[test]
    fn test_valid_header() {
        let mut data = Vec::new();
        data.extend_from_slice(b"II"); // Byte order
        data.extend_from_slice(&26u32.to_le_bytes()); // Header length
        data.extend_from_slice(CIFF_SIGNATURE); // Valid signature
        data.extend_from_slice(&[0u8; 10]); // Version + reserved
        data.extend_from_slice(&100u32.to_le_bytes()); // Root dir offset

        let cursor = Cursor::new(data);
        let result = CiffParser::new(cursor);

        assert!(result.is_ok());
        let parser = result.unwrap();
        assert_eq!(parser.header.root_dir_offset, 100);
    }
}
